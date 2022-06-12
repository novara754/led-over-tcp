#include "tcp.h"

#include "sdkconfig.h"

#include "driver/gpio.h"
#include "esp_event.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_wifi.h"
#include "nvs_flash.h"

#include "lwip/err.h"
#include "lwip/sockets.h"
#include "lwip/sys.h"
#include <lwip/netdb.h>

static const char *TAG = "led_over_tcp:tcp";

static void handle_client(const int sock)
{
    int len;
    char rx_buffer[128];

    do
    {
        len = recv(sock, rx_buffer, sizeof(rx_buffer) - 1, 0);
        if (len < 0)
        {
            ESP_LOGE(TAG, "Failed to receive data from client: errno %d", errno);
        }
        else if (len == 0)
        {
            ESP_LOGW(TAG, "Connection closed");
        }
        else
        {
            rx_buffer[len] = 0;
            ESP_LOGI(TAG, "Received %d bytes: '%s'", len, rx_buffer);

            int to_write = len;
            while (to_write > 0)
            {
                int written = send(sock, rx_buffer, to_write, 0);
                if (written < 0)
                {
                    ESP_LOGE(TAG, "Failed to send data to client: errno %d", errno);
                    break;
                }
                to_write -= written;
            }
        }
    }
    while (len > 0);
}

void tcp_server_task(void *args)
{
    (void)args;

    ESP_LOGI(TAG, "Starting TCP server...");

    int ip_protocol = IPPROTO_IP;
    struct sockaddr_storage dest_addr;
    {
        struct sockaddr_in *dest_addr_ip4 = (struct sockaddr_in *)&dest_addr;
        dest_addr_ip4->sin_addr.s_addr = htonl(INADDR_ANY);
        dest_addr_ip4->sin_family = AF_INET;
        dest_addr_ip4->sin_port = htons(CONFIG_TCP_PORT);
    }

    int listen_sock = socket(AF_INET, SOCK_STREAM, ip_protocol);
    if (listen_sock < 0)
    {
        ESP_LOGE(TAG, "Failed to create socket: %d", errno);
        vTaskDelete(NULL);
        return;
    }

    int opt = 1;
    setsockopt(listen_sock, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    ESP_LOGI(TAG, "TCP socket created");

    {
        int err = bind(listen_sock, (struct sockaddr *)&dest_addr, sizeof(dest_addr));
        if (err != 0)
        {
            ESP_LOGE(TAG, "Failed to bind socket: %d", errno);
            goto CLEANUP;
        }
    }

    ESP_LOGI(TAG, "Socket bound to port %d", CONFIG_TCP_PORT);

    {
        int err = listen(listen_sock, 1);
        if (err != 0)
        {
            ESP_LOGE(TAG, "Failed to listen to incoming connection: %d", errno);
            goto CLEANUP;
        }
    }

    while (1)
    {
        ESP_LOGI(TAG, "Waiting for incoming connection");

        struct sockaddr_storage source_addr;
        socklen_t addr_len = sizeof(source_addr);

        int client_sock = accept(listen_sock, (struct sockaddr *)&source_addr, &addr_len);
        if (client_sock < 0)
        {
            ESP_LOGE(TAG, "Failed to accept incoming connection: %d", errno);
            break;
        }

        int keep_alive = 1;
        int keep_idle = CONFIG_KEEPALIVE_IDLE;
        int keep_interval = CONFIG_KEEPALIVE_INTERVAL;
        int keep_count = CONFIG_KEEPALIVE_COUNT;
        setsockopt(client_sock, SOL_SOCKET, SO_KEEPALIVE, &keep_alive, sizeof(keep_alive));
        setsockopt(client_sock, IPPROTO_TCP, TCP_KEEPIDLE, &keep_idle, sizeof(keep_idle));
        setsockopt(client_sock, IPPROTO_TCP, TCP_KEEPINTVL, &keep_interval, sizeof(keep_interval));
        setsockopt(client_sock, IPPROTO_TCP, TCP_KEEPCNT, &keep_count, sizeof(keep_count));

        char addr_str[128] = {0};
        if (source_addr.ss_family == PF_INET)
        {
            inet_ntoa_r(
                ((struct sockaddr_in *)&source_addr)->sin_addr.s_addr,
                addr_str,
                sizeof(addr_str) - 1
            );
        }

        ESP_LOGI(TAG, "Accepted incoming connection from %s", addr_str);

        handle_client(client_sock);

        shutdown(client_sock, 0);
        close(client_sock);
    }

CLEANUP:
    close(listen_sock);
    vTaskDelete(NULL);
}

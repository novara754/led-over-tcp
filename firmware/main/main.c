#include <stdio.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "driver/gpio.h"
#include "esp_log.h"
#include "esp_system.h"
#include "nvs_flash.h"

#include "tcp.h"
#include "wifi.h"

static const char *TAG = "led_over_tcp:main";

static void configure_led(void)
{
    gpio_reset_pin(CONFIG_BLINK_GPIO);
    gpio_set_direction(CONFIG_BLINK_GPIO, GPIO_MODE_OUTPUT);
}

void app_main(void)
{
    ESP_ERROR_CHECK(nvs_flash_init());

    ESP_LOGI(TAG, "Initializing GPIO pins...");
    configure_led();

    ESP_LOGI(TAG, "Initializing WIFI...");
    wifi_init_sta();

    xTaskCreate(tcp_server_task, "tcp_server", 4096, NULL, 5, NULL);
}

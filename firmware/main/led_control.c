#include "led_control.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "driver/gpio.h"
#include "esp_log.h"

#include "sdkconfig.h"

static const char *TAG = "led_over_tcp:led_control";

static TaskHandle_t s_led_control_task_handle = NULL;
static bool s_led_level = false;

static void configure_led(void)
{
    gpio_reset_pin(CONFIG_BLINK_GPIO);
    gpio_set_direction(CONFIG_BLINK_GPIO, GPIO_MODE_OUTPUT);
}

static void task(void *arg)
{
    s_led_control_task_handle = xTaskGetCurrentTaskHandle();

    ESP_LOGI(TAG, "Initializing GPIO pins...");
    configure_led();

    while (true)
    {
        if (ulTaskNotifyTake(pdFALSE, portMAX_DELAY))
        {
            s_led_level = !s_led_level;
            ESP_LOGI(TAG, "Setting new LED level");
            gpio_set_level(CONFIG_BLINK_GPIO, s_led_level);
        }
    }
}

void led_control_start(void)
{
    ESP_LOGI(TAG, "Starting led_control task");
    xTaskCreate(task, "led_control", 4096, NULL, 5, NULL);
}

bool led_control_toggle(void)
{
    if (!s_led_control_task_handle)
    {
        ESP_LOGE(TAG, "Received command to toggle LED while task not running");
        return false;
    }

    ESP_LOGI(TAG, "Received command to toggle LED");
    bool new_level = !s_led_level;
    xTaskNotifyGive(s_led_control_task_handle);
    return new_level;
}

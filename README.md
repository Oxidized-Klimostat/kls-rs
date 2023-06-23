# Oxidized Klimostat

This is a revival of a [school project](https://github.com/Klimostat) from a few years ago.

Initiated in the midst of the 2020 coronavirus pandemic, the project aimed to establish an air quality monitoring station in every classroom. The purpose  was to alert when poor air quality was detected, prompting the need to open windows and ventilate the classrooms. The project was originally written in Python, which isn't the best language for embedded development, so there were issues regarding reliability.

## Hardware

This project uses an **[ESP32-C3](https://www.espressif.com/en/products/socs/esp32-c3)** microcontroller and the **[SCD30](https://sensirion.com/products/catalog/SCD30)** sensor.

The ESP32-C3 is a microcontroller using the RISC-V instruction set.

The Sensirion SCD30 sensor can measure CO<sub>2</sub>, humidity and temperature with high accuracy. The microcontroller uses the I<sup>2</sup>C-protocol to communicate with the sensor.

## Status

This project is still in its very early stages of development, breaking changes are expected.

## License

GPLv3

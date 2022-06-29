# Briteboard

I put some LEDs under an acrylic sheet, then went _way_ overboard with it.

## Requirements

- Rust nightly
- Teensy-loader-cli or equivalent
- PlatformIO

## Hardware and Architecture

- Teensy 4.x
- WS2812 LED strip (neopixels)
- ssd1351 OLED screen
- NodeMCUv2 (ESP8266 wifi)
- Rotary encoders for input

It all started with a teensy connected to an LED strip. 
Of course, I wanted to add controls to it in the form of rotary encoders and a screen. 
Turns out it's not that easy. To effectively use a rotary encoder, you need to frequently query it. 
That's not so easy when you spend 95+% of loop time updating the screen.
Alternatively, rotary encoder inputs could trigger interrupts... interrupting the screen and LED drawing.
Free seizures for all.

So I say, screw it, let's add a second board just for grabbing inputs, and we can query that whenever we're not busy.
May as well make it wireless so you can control it from your phone or something, too. 
That's where the NodeMCUv2 comes in. 
Unfortunately you can't* program these in Rust yet, so we're using the arduino libraries via PlatformMCU.
As a bonus, we can theoretically apply OTA updates to NodeMCU 
(and possibly from there, the teensy, but that's way beyond the scope of what I want to do).

\* Okay, you _kind of_ can, but it's very experimental - and library support would suck anyway.

## Development

### Rust + Teensy

Build the ihex artifact  
`cargo objcopy --release -- -O ihex briteboard.hex`

Upload with `teensy-loader-cli` or your loader of choice  
`teensy_loader_cli --mcu=imxrt1062 -w briteboard.hex`

### PlatformIO + NodeMCUv2

`cd control-board`  
`pio run -t upload`  

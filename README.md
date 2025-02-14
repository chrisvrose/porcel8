# PORCEL-8

Chip 8 emulator/interpreter.

```bash
./porcel8 an_awesome_chip8_rom.ch8
```

![pong.gif](assets/pong.gif)


Please refer to the [Relevant Resources](#relevant-resources) section for some publicly available ROMs.


### Status

<details>
<summary>Implementation status</summary>

- [X] Memory
- [X] Timer
- [X] Loading font
  - [X] Default font
  - [ ] ~~Custom font~~ Future
- [X] Registers
- [X] Stack
- [X] Display
- [X] Instruction Processing
  - [X] Bare requirements for IBM Logo
  - [X] ALU operations
  - [X] Procedure related
  - [X] Timer
  - [X] Super chip8 compatibility.
- [X] Audio
  - Audio seems to stutter, but working
- [X] Keyboard

</details>

Known inaccuracies:
- Get key is triggered when key is pressed (not just released)
- Slight display and audio stutters
  - Audio stutters are due to a workaround needed from an issue with SDL2 in my system.

### Relevant Resources

- [Guide to making a CHIP-8 emulator - Tobias V. Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#specifications)
- [CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite)
- [CHIP-8 Program Pack](https://github.com/kripod/chip8-roms)
- [Awesome CHIP-8](https://chip-8.github.io/links/)

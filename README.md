# PORCEL-8

Chip 8 emulator/interpreter.

![pong.gif](assets/pong.gif)

There are no included ROMs as part of this project. Please refer to the [Relevant Resources](#relevant-resources) section.

```bash
./porcel8 -f your_rom.ch8
```


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
- Display stutters
  - This is due to rendering happening on a separate thread.
- Audio stutters
  - This is due to using an audio queue, instead of the traditional audio callback.

### Relevant Resources

- [Guide to making a CHIP-8 emulator - Tobias V. Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#specifications)
- [CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite)
- [CHIP-8 Program Pack](https://github.com/kripod/chip8-roms)
- [Awesome CHIP-8](https://chip-8.github.io/links/)

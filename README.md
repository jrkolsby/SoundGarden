# PalIt

Palit is a music hardware project which seeks to upcycle outdated laptop models into audio workstations for collaborative music production over IP. 

### Projects
1. User Interface (UI) - All things relating to the ncurses interface.
2. Hardware (HW) - All things relating to peripherals and hardware modifications to the laptop.
3. Design (DG) - All things related to mockups, layouts, and the customization of the hardware appearance beyond its functionality.
4. Sound Engine (SOUND) - All things related to the sampling, recording, and synthesis engine including integration with Faust and custom patches
5. Infrastructure (INF) - All things relating to the structure of this codebase including our testing suite, issue tracking, and the configuration of our Ubuntu OS and all required software. This includes the development environment in Ubuntu and the flashing of installable media for use on our laptops.  

## Sprints

### OCTOBER 1

- [X] SOUND-1: build a dsp file
- [X] SOUND-2: Implement play and stop to ALSA output
- [X] SOUND-5: Multitrack mixing
- [X] SOUND-6: Receive Midi events in pt-sound

- [X] UI-1: Display the splash graphic
- [X] UI-2: Splash screen with project listings
- [X] UI-4: Display empty timeline for new project
- [X] UI-5: Timeline Cursor
- [X] UI-6: Layered rendering
- [X] UI-8: Multi Cursor
- [X] UI-9: Receive ticks from pt-sound playback

- [X] HARD-1: Acquire Lenovo Thinkpad T400

- [X] DG-1: First iteration mockup for user flow 
- [X] DG-1: Acquire vinyl paints in four-function colors and SELECT RED
- [X] DG-2: Paint top keybed using glossy white and black nail polish
- [X] DG-4: Paint bottom keybed

- [X] INF-2: Design project filetype around .xml or .json (Hopefully compatible with android)
- [X] INF-3: Implement view state
- [X] INF-4: Remove cursive dep in favor of termion
- [X] INF-7: Debug console
- [X] INF-8: Read XML file and update timeline state
- [X] INF-9: Update pt-client to loop which polls sdl2 keyboard and /tmp/pt-sound

### NOVEMBER 1

- [X] SOUND-9: Root mixer for timelines and synth (DSP Graph)
- [X] SOUND-11: Arpeggio
- [X] SOUND-12: Chord Gen

### DECEMBER 1

- [X] UI-20: Refactor components to be color-independent
- [X] UI-10: Render partial waveforms
- [X] UI-18: Routes view
- [X] UI-19: Timeline editing

### JANUARY 1 2020

- [X] UI-21: Add route, patch io
- [ ] SOUND-13: Timeline looping
- [ ] SOUND-14: Timeline editing
- [ ] SOUND-15: Timeline XML save/load (BIG BRANCH)

### FEBRUARY 1 2020

- [ ] INF-5: Unicode Support in linux TTY with KMSCON

- [ ] UI-17: Add module modal
- [ ] SOUND-10: Midi Tape

- [ ] SOUND-20: Import audio file to timeline

### MARCH 1 ( v1.0 )

- [ ] SOUND-3: Faust synth compilation
- [ ] SOUND-4: ALSA input module
- [ ] SOUND-19: Timeline recording

- [ ] INF-10: Faust UI generation

### APRIL 1

- [ ] UI-14: Network setup
- [ ] SOUND-18: Radio module and directory API

### MAY 1

- [ ] SOUND-7: Send midi events from pt-client
- [ ] SOUND-8: Fix audio underruns
- [ ] SOUND-16: VST audio host (Steinberg API)
- [ ] SOUND-17: VST client host (XML UI to text layout)

- [ ] UI-7: Loading screen
- [ ] UI-12: Error Screen
- [ ] UI-15: Keyboard setup

- [ ] HARD-3: Purchase functional android phone
- [ ] HARD-4: Replace laptop sreen with android phone replacement screen
- [ ] HARD-5: Document undetectable keybed note combinations and map to chords
- [ ] HARD-6: Determine a set of keys which require redundant connections to unused keys in order to maximize chord combinations for the top (polyphonic) keybed.
- [ ] HARD-7: Encoder usage with function keys

- [ ] DG-3: Paint four function keys
- [ ] DG-5: Paint laptop casing and general keys CLASSIC BEIGE

- [ ] INF-1: Implement testing suite which takes keyboard input and project files and outputs UI sequences and stored audio files

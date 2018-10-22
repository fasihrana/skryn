# skryn

## What is skryn?

skryn is a (work-in-progress) Desktop GUI framework based on [servo/webrender](https://github.com/servo/webrender). It's aim is to have a pure rust implementation of a framework that is easy to use and extend. The motivation behind starting skryn was to get a simple hardware based rendered GUI without having any other web technologies be in the way.

## Goals/Features

1. Start the window manager at desired FPS.
2. The minimum requirement to create your own elements is to implement the `Element` trait.
3. Use implemented elements to create complex elements.
4. Library provided minimalistic `Observable`s. 
5. Multithreading safe.
6. Simplified length Units (Natural, Extent, Stretch, Pixel).
7. Show a cursor in `TextBox` element.
8. Paste into and Copy from `TextBox`.

## Project Status (Limitations/Features planned)

There are many limitations in the project. Following is the known list of these (not limited to)

1. Cross Element communication
2. Observables need a better implementation.
3. Use [pilcrow](https://github.com/pcwalton/pilcrow) for paragraph rendering.
4. There are no animations at the moment. (Possible through implementation of own Element).
5. `ElementEvent` needs more events like HoverBegin,HoverEnd.
6. Needs z-index like concept.
7. Support for RTL languages.

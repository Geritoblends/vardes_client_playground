# Client plugin engine

<!-- We're building a plugin engine where all its plugins can be both, a WASM module or a dynamic library, in a way that they're both unified as Plugins. -->

We're now building a todo app that has its core logic inside a wasm module and a wasm module produces side effects after listening to the core's events.
Then, after having WASM logic correct, we'll add dynamic libraries and finally we might start making the client plugin engine for the mmo

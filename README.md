# sleepyhead-kdl
an evented & optionally no-std parser for [KDL](https://kdl.dev/)

### differences with kdl-rs
#### Events
sleepyhead-kdl uses an evented parser; which provide a stream-like interface.

this has the pros of being really fast and allowing for flexibility in how you consume your kdl, at the cost of some convenience for a lot of typical use cases.
#### no-std
sleepyhead-kdl supports no-std! 
the no-std version is somewhat slower on account of using `heapless` Vec's, but it's still workable!
#### zero-copy-ish
as a challenge, i tried to make this as zero-copy as possible. while some copying can't be avoided - like in processing string escapes - most of it is lazy. things like raw strings are fully zero-copy!

### feature flags
- std: enables std support (on by default)
- alloc: enables alloc support in no-std environments 

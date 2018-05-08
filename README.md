# jakar-threadpool
A small utility crate for the [jakar engine](https://github.com/SiebenCorgie/jakar-engine).

## Functionality
The `ThreadPool` can spawn several threads and distributes the asynchronous over
a communication channel to the first thread which needs new work.
The job currently has to be a function with the signature `FnOnce() + Send`.
However, I plan to implement other signatures when needed in the future or move
the system to trait based system.

## License
You can decide if you want to use MIT or Apache License, Version 2.0.

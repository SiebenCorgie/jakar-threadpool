# jakar-threadpool
A small utility crate for the [jakar engine](https://gitlab.com/Siebencorgie/jakar-engine).

## MOVED TO GITLAB

Since I am kind of a idealist I moved the whole project including the sub crates to Gitlab:
https://gitlab.com/Siebencorgie/jakar-engine

## Functionality
The `ThreadPool` can spawn several threads and distributes the asynchronous over
a communication channel to the first thread which needs new work.
The job currently has to be a function with the signature `FnOnce() + Send`.
However, I plan to implement other signatures when needed in the future or move
the system to trait based system.

## License
You can decide if you want to use MIT or Apache License, Version 2.0.

# multi-language-communication
A project to run multiple languages in a system


# Run codes

To run the code for Shared Memory examples:

```
python3 src/main.py
```

In this project, we achieved a communication between a python application and a js app through shared memory.
Further implementations can be done to test communication between other application.

The communication link is written in rust and compiled. The message to a function call is made through shared memory. It's written in a queue by a caller, and the other side is a listener which listens for a call function message. When a message is seen, the code runs the function with it's parameters and writes the outputs in another queue.

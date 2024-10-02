# Multi-Language Communication

This project facilitates communication between applications written in different programming languages using shared memory. It demonstrates an efficient method for message passing between Python and JavaScript, with a Rust-based interface managing the communication.

## Features

- **Cross-language communication**: Seamless interaction between Python and JavaScript.
- **Shared memory queues**: Efficient message passing using shared memory.
- **Rust-based middleware**: Bridges Python and JavaScript communication.
- **Extensible**: Adaptable for adding more languages.

## Prerequisites

- Python 3.x
- Node.js 14.x or higher
- Cargo (Rust package manager)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/SobhanMoghimi/multi-language-communication.git

2. Navigate to the project directory:

   ```bash
    cd multi-language-communication

3. Install Python dependencies:

    ```bash
    pip install -r requirements.txt
 
4. Install Node.js dependencies:

   ```bash
    npm install

5. Build the Rust components:

   ```bash
   cargo build

## Usage
Start the Python and Node.js services:
 
    npm start


Messages will be passed between the Python and JavaScript applications using the Rust middleware.


## Contributing
Feel free to open issues or submit pull requests to contribute to the project.

## License
This project is licensed under the MIT License. See the LICENSE file for details.

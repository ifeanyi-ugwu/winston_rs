use winston::logger_v2::{
    transport::{create_console_transport, create_file_transport, initialize_runtime},
    Logger,
};

#[test]
fn test_basic_usage() {
    initialize_runtime();

    let console_transport = create_console_transport();
    let console_logger = Logger::new(console_transport);

    console_logger.log("This is a message to the console");

    // Create a file logger
    let file_transport = create_file_transport("log.txt");
    let file_logger = Logger::new(file_transport);

    file_logger.log("This is a message to the file");
}

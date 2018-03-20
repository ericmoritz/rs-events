set -e
diesel setup
diesel migration run
target/release/test_server

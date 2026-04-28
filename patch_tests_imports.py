import re

with open("src/matrix/tests.rs", "r") as f:
    content = f.read()

# Add wiremock imports inside the functions
content = content.replace("async fn test_leave_room_success() {", "async fn test_leave_room_success() {\n    use wiremock::{Mock, MockServer, ResponseTemplate};\n    use wiremock::matchers::{method, path_regex};")

content = content.replace("async fn test_leave_room_error() {", "async fn test_leave_room_error() {\n    use wiremock::{Mock, MockServer, ResponseTemplate};\n    use wiremock::matchers::{method, path_regex};")

with open("src/matrix/tests.rs", "w") as f:
    f.write(content)

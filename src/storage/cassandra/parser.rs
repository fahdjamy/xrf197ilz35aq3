/// ParserState: Represents the current parsing state while iterating through the CQL file.
enum ParseState {
    /// Default state, processing regular CQL.
    Default,
    /// Inside a single-quoted string literal ('...').
    InString,
    /// Inside a single-line comment (-- ...).
    InSingleLineComment,
    /// Inside a multi-line comment (/* ... */).
    InMultiLineComment,
}

fn parse_cql_statements(cql_context: &str) -> Vec<String> {
    let mut state = ParseState::Default;
    let mut statements = Vec::new();
    let mut current_statement = String::new();

    let mut chars = cql_context.chars().peekable();

    while let Some(c) = chars.next() {
        match state {
            ParseState::Default => {
                if c == '\'' {
                    state = ParseState::InString;
                    current_statement.push(c);
                } else if c == '-' && chars.peek() == Some(&'-') {
                    state = ParseState::InSingleLineComment;
                    // We can choose to preserve or discard comments. Currently, we discard.
                    chars.next(); // Consume the second '-'
                } else if c == '/' && chars.peek() == Some(&'*') {
                    state = ParseState::InMultiLineComment;
                    // Discard comments.
                    chars.next(); // Consume the '*'
                } else if c == ';' {
                    // End of a statement found.
                    if !current_statement.trim().is_empty() {
                        statements.push(current_statement.trim().to_string());
                    }
                    current_statement.clear();
                } else {
                    current_statement.push(c);
                }
            }
            ParseState::InString => {
                current_statement.push(c);
                if c == '\'' {
                    // Cassandra uses '' to escape a single quote inside a string.
                    if chars.peek() == Some(&'\'') {
                        // This is an escaped quote, consume the next one as well.
                    } else {
                        state = ParseState::Default;
                    }
                }
            }
            ParseState::InSingleLineComment => {
                if c == '\n' {
                    // End of comment, return to default state.
                    // Push the newline to maintain line breaks between statements if desired.
                    current_statement.push(c);
                    state = ParseState::Default;
                }
            }
            ParseState::InMultiLineComment => {
                if c == '*' && chars.peek() == Some(&'/') {
                    // End of multi-line comment.
                    chars.next(); // Consume the '/'
                    state = ParseState::Default;
                }
            }
        }
    }

    // Add the last statement if the file doesn't end with a semicolon.
    if !current_statement.trim().is_empty() {
        statements.push(current_statement.trim().to_string());
    }

    statements
}

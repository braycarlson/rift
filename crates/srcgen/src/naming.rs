const ACRONYM_RUN_LENGTH_MIN: usize = 3;
const IDENT_LENGTH_MAX: usize = 256;

pub fn const_name(label: &str) -> String {
    assert!(!label.is_empty(), "label must not be empty");
    assert!(
        label.len() <= IDENT_LENGTH_MAX,
        "label exceeds {IDENT_LENGTH_MAX}: {label}"
    );

    let mut result = String::with_capacity(label.len());

    for character in label.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_uppercase());
        } else {
            result.push('_');
        }
    }

    assert!(!result.is_empty(), "const name must not be empty: {label}");
    assert!(
        !result.starts_with(|c: char| c.is_ascii_digit()),
        "const name must not start with digit: {label}",
    );

    result
}

pub fn field_name(property_name: &str) -> String {
    assert!(!property_name.is_empty(), "property name must not be empty");
    assert!(
        property_name.len() <= IDENT_LENGTH_MAX,
        "property name exceeds {IDENT_LENGTH_MAX}: {property_name}",
    );

    let characters: Vec<char> = property_name.chars().collect();
    let mut result = String::with_capacity(characters.len() + 8);

    for index in 0..characters.len() {
        let character = characters[index];

        if !character.is_ascii_uppercase() {
            if character.is_ascii_alphanumeric() {
                result.push(character);
            } else {
                result.push('_');
            }

            continue;
        }

        if index > 0 {
            let previous = characters[index - 1];
            let boundary_word = previous.is_ascii_lowercase() || previous.is_ascii_digit();
            let boundary_acronym = previous.is_ascii_uppercase()
                && index + 1 < characters.len()
                && characters[index + 1].is_ascii_lowercase();

            if boundary_word || boundary_acronym {
                result.push('_');
            }
        }

        result.push(character.to_ascii_lowercase());
    }

    assert!(
        !result.is_empty(),
        "field name must not be empty: {property_name}"
    );

    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, 'x');
    }

    assert!(
        !result.starts_with(|c: char| c.is_ascii_digit()),
        "field name must not start with digit: {property_name}",
    );

    result
}

pub fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "union"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

pub fn module_name(prefix: &str) -> String {
    assert!(!prefix.is_empty(), "prefix must not be empty");
    assert!(
        prefix.len() <= IDENT_LENGTH_MAX,
        "prefix exceeds {IDENT_LENGTH_MAX}: {prefix}"
    );

    let mut result = String::with_capacity(prefix.len());

    for character in prefix.chars() {
        assert!(
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-',
            "unexpected prefix character in: {prefix}",
        );

        if character == '-' {
            result.push('_');
        } else {
            result.push(character);
        }
    }

    assert!(
        !result.starts_with(|c: char| c.is_ascii_digit()),
        "module name must not start with digit: {prefix}",
    );

    result
}

pub fn pascal_name(snake: &str) -> String {
    assert!(!snake.is_empty(), "snake name must not be empty");
    assert!(
        snake.len() <= IDENT_LENGTH_MAX,
        "snake name exceeds {IDENT_LENGTH_MAX}: {snake}"
    );

    let mut result = String::with_capacity(snake.len());
    let mut uppercase_next = true;

    for character in snake.chars() {
        assert!(
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_',
            "unexpected snake name character in: {snake}",
        );

        if character == '_' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            result.push(character.to_ascii_uppercase());
        } else {
            result.push(character);
        }

        uppercase_next = false;
    }

    assert!(!result.is_empty(), "pascal name must not be empty: {snake}");

    result
}

pub fn struct_name(raw_name: &str) -> String {
    assert!(!raw_name.is_empty(), "struct name must not be empty");
    assert!(
        raw_name.len() <= IDENT_LENGTH_MAX,
        "struct name exceeds {IDENT_LENGTH_MAX}: {raw_name}"
    );

    let characters: Vec<char> = raw_name.chars().collect();
    let mut result = String::with_capacity(characters.len());
    let mut index: usize = 0;

    while index < characters.len() {
        let character = characters[index];

        assert!(
            character.is_ascii_alphanumeric(),
            "unexpected struct name character: {raw_name}"
        );

        if !character.is_ascii_uppercase() {
            result.push(character);
            index += 1;
            continue;
        }

        let mut run_end = index;

        while run_end < characters.len() && characters[run_end].is_ascii_uppercase() {
            run_end += 1;
        }

        struct_name_run(&mut result, &characters, index, run_end);

        index = run_end;
    }

    assert!(
        !result.is_empty(),
        "struct name must not be empty: {raw_name}"
    );

    let first = result.as_bytes()[0] as char;

    if first.is_ascii_lowercase() {
        result.replace_range(0..1, &first.to_ascii_uppercase().to_string());
    }

    assert!(
        result.starts_with(|c: char| c.is_ascii_uppercase()),
        "struct name must start uppercase: {raw_name}",
    );

    result
}

fn struct_name_run(result: &mut String, characters: &[char], index: usize, run_end: usize) {
    assert!(index < run_end, "run must be non-empty");
    assert!(run_end <= characters.len(), "run end must be within bounds");

    let run_length = run_end - index;
    let run_continues_word = run_end < characters.len() && characters[run_end].is_ascii_lowercase();

    if run_length >= ACRONYM_RUN_LENGTH_MIN {
        let keep_upper_tail = usize::from(run_continues_word);

        result.push(characters[index]);

        for &lowered in &characters[(index + 1)..(run_end - keep_upper_tail)] {
            result.push(lowered.to_ascii_lowercase());
        }

        for &kept in &characters[(run_end - keep_upper_tail)..run_end] {
            result.push(kept);
        }
    } else {
        for &kept in &characters[index..run_end] {
            result.push(kept);
        }
    }
}

pub fn variant_name(key: &str) -> String {
    assert!(!key.is_empty(), "route key must not be empty");
    assert!(
        key.len() <= IDENT_LENGTH_MAX,
        "route key exceeds {IDENT_LENGTH_MAX}: {key}"
    );

    for character in key.chars() {
        assert!(
            character.is_ascii_lowercase() || character.is_ascii_digit(),
            "unexpected route key character in: {key}",
        );
    }

    let first = key.as_bytes()[0] as char;

    assert!(
        first.is_ascii_lowercase(),
        "route key must start with letter: {key}"
    );

    let mut result = String::with_capacity(key.len());

    result.push(first.to_ascii_uppercase());
    result.push_str(&key[1..]);

    result
}

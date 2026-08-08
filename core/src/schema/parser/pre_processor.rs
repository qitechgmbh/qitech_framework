use std::collections::VecDeque;

enum Line {
    Process(String),
    Emit(String),
}

pub fn pre_process(s: &str) -> String {
    let mut lines: VecDeque<Line> = s
        .lines()
        .map(|line| Line::Process(line.to_owned()))
        .collect();

    let mut output = String::new();

    while let Some(line) = lines.pop_front() {
        match line {
            Line::Emit(line) => {
                output.push_str(&line);
                output.push('\n');
            }

            Line::Process(line) => {
                let generated = process_line(&line, &mut lines);

                // Preserve generated order.
                for line in generated.into_iter().rev() {
                    lines.push_front(line);
                }
            }
        }
    }

    output
}

fn process_line(line: &str, lines: &mut VecDeque<Line>) -> Vec<Line> {
    let indent = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();

    let Some((_, value)) = trimmed.split_once(':') else {
        return vec![Line::Emit(line.to_owned())];
    };

    let value = value.trim_start();

    let list = match value {
        value if value.starts_with("!?list ") => "!?list",
        value if value.starts_with("!list ") => "!list",
        _ => return vec![Line::Emit(line.to_owned())],
    };

    let rest = value[list.len()..].trim_start();

    if !rest.starts_with('!') {
        return vec![Line::Emit(line.to_owned())];
    }

    // Preserve the original line up to and including !list.
    let list_pos = line.find(list).unwrap();
    let current = line[..list_pos + list.len()].to_owned();

    let item_line = format!("{}item: {}", " ".repeat(indent + 2), rest,);

    let next_type = rest.split_whitespace().next().unwrap();

    let next_type = next_type
        .strip_prefix("!?")
        .or_else(|| next_type.strip_prefix('!'))
        .unwrap_or(next_type);

    let block = match next_type {
        // disabled due to complexity and we should focus on one canonical layout
        // "object" if rest.contains('{') => Some(collect_flow_block(lines, '}')),
        "object" => Some(collect_indented_block(lines)),
        "enum" if rest.contains('[') => Some(collect_flow_block(lines, ']')),
        "enum" => Some(collect_indented_block(lines)),
        _ => None,
    };

    match block {
        Some(block) => {
            let block = reindent_block(block, indent + 4);

            with_block(current, item_line, block)
        }

        None => {
            vec![Line::Emit(current), Line::Process(item_line)]
        }
    }
}

fn with_block(current: String, item_line: String, block: Vec<String>) -> Vec<Line> {
    let mut result = Vec::with_capacity(block.len() + 2);

    result.push(Line::Emit(current));
    result.push(Line::Process(item_line));

    result.extend(block.into_iter().map(Line::Process));

    result
}

fn collect_flow_block(lines: &mut VecDeque<Line>, closing: char) -> Vec<String> {
    let mut block = Vec::new();

    while let Some(line) = lines.pop_front() {
        let Line::Process(line) = line else {
            unreachable!();
        };

        let done = line.contains(closing);

        block.push(line);

        if done {
            break;
        }
    }

    block
}

fn collect_indented_block(lines: &mut VecDeque<Line>) -> Vec<String> {
    let mut block = Vec::new();

    let base_indent = loop {
        let Some(Line::Process(line)) = lines.front() else {
            return block;
        };

        if line.trim().is_empty() {
            let Line::Process(line) = lines.pop_front().unwrap() else {
                unreachable!();
            };

            block.push(line);
            continue;
        }

        break line.len() - line.trim_start().len();
    };

    while let Some(Line::Process(line)) = lines.front() {
        if !line.trim().is_empty() {
            let indent = line.len() - line.trim_start().len();

            if indent < base_indent {
                break;
            }
        }

        let Line::Process(line) = lines.pop_front().unwrap() else {
            unreachable!();
        };

        block.push(line);
    }

    block
}

fn reindent_block(block: Vec<String>, target_indent: usize) -> Vec<String> {
    let base_indent = block
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .unwrap_or(0);

    block
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                let indent = line.len() - line.trim_start().len();
                let relative = indent.saturating_sub(base_indent);

                format!(
                    "{}{}",
                    " ".repeat(target_indent + relative),
                    line.trim_start(),
                )
            }
        })
        .collect()
}

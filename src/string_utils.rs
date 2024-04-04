#[inline]
pub fn convert_str_to_underscore_case(s: &str) -> String {
    let mut l: char = '-';
    s.chars()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i > 0 && c.is_uppercase() && !l.is_whitespace() {
                // (c.is_uppercase() || c.is_whitespace()) {
                acc.push('_');
            }
            l = c;
            acc.push(c);
            acc
        })
        .to_lowercase()
}

#[inline]
pub fn convert_str_to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            word.chars()
                .enumerate()
                .fold(String::new(), |mut acc, (i, c)| {
                    if i == 0 {
                        acc.push_str(&c.to_uppercase().to_string());
                    } else {
                        acc.push_str(&c.to_lowercase().to_string());
                    }
                    acc
                })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

//
//
//
#[test]
fn test_to_title_case() {
    let input_case = "this is a stRing to conVert to title case.";
    let title_case = convert_str_to_title_case(input_case);
    let expected_case = "This Is A String To Convert To Title Case.";
    assert_eq!(title_case, expected_case);
}

#[test]
fn test_to_underscore_case() {
    let input_case = "this is a stRing to conVert to UnderScore case.";
    let underscore_case = convert_str_to_underscore_case(input_case);
    let expected_case = "this is a st_ring to con_vert to under_score case.";
    assert_eq!(underscore_case, expected_case);
}

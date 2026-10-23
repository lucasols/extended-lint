mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "use-state-setter-naming";

#[test]
fn valid_usestate_destructuring_with_correct_naming() {
    valid(
        RULE,
        indoc! {r#"
        const [count, setCount] = useState(0);
    "#},
    );
}

#[test]
fn use_without_setter() {
    valid(
        RULE,
        indoc! {r#"
        const [count] = useState(0);
    "#},
    );
}

#[test]
fn valid_usestate_destructuring_with_camelcase_naming() {
    valid(
        RULE,
        indoc! {r#"
        const [userName, setUserName] = useState('');
    "#},
    );
}

#[test]
fn valid_usestate_destructuring_with_abbreviation() {
    valid(
        RULE,
        indoc! {r#"
        const [isLoading, setIsLoading] = useState(false);
    "#},
    );
}

#[test]
fn valid_usestate_destructuring_with_underscore() {
    valid(
        RULE,
        indoc! {r#"
        const [user_data, setUser_data] = useState({});
    "#},
    );
}

#[test]
fn valid_usestate_destructuring_with_numbers() {
    valid(
        RULE,
        indoc! {r#"
        const [step1, setStep1] = useState(0);
    "#},
    );
}

#[test]
fn invalid_usestate_destructuring_with_incorrect_setter_name() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [records, setGroups] = useState([]);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setRecords" with a "set" prefix but got "setGroups"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [records, setRecords] = useState([]);");
}

#[test]
fn invalid_usestate_destructuring_with_completely_different_name() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [data, updateItems] = useState([]);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setData" with a "set" prefix but got "updateItems"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [data, setData] = useState([]);");
}

#[test]
fn invalid_usestate_destructuring_with_wrong_prefix() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [user, changeUser] = useState(null);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setUser" with a "set" prefix but got "changeUser"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [user, setUser] = useState(null);");
}

#[test]
fn valid_react_usestate_usage() {
    valid(
        RULE,
        indoc! {r#"
        const [items, setItems] = React.useState([]);
    "#},
    );
}

#[test]
fn invalid_react_usestate_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [items, updateItems] = React.useState([]);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setItems" with a "set" prefix but got "updateItems"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [items, setItems] = React.useState([]);");
}

#[test]
fn ignores_non_usestate_calls() {
    valid(
        RULE,
        indoc! {r#"
        const [a, b] = someFunction();
        const [x, y] = useCustomHook();
    "#},
    );
}

#[test]
fn ignores_usestate_with_single_destructuring() {
    valid(
        RULE,
        indoc! {r#"
        const [value] = useState('test');
    "#},
    );
}

#[test]
fn ignores_usestate_with_more_than_two_destructuring_elements() {
    valid(
        RULE,
        indoc! {r#"
        const [value, setValue, extra] = useState('test');
    "#},
    );
}

#[test]
fn handles_complex_camelcase_names_correctly() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [userProfileData, updateProfile] = useState({});
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setUserProfileData" with a "set" prefix but got "updateProfile"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [userProfileData, setUserProfileData] = useState({});");
}

#[test]
fn valid_with_ignoreprefixes_underscore_prefix_in_setter() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const [value, _setValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
}

#[test]
fn valid_with_ignoreprefixes_underscore_prefix_in_value() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const [_value, setValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
}

#[test]
fn valid_with_ignoreprefixes_underscore_in_both() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const [_value, _setValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
}

#[test]
fn valid_with_ignoreprefixes_single_underscore_value() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const [_, setValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
}

#[test]
fn valid_with_ignoreprefixes_multiple_prefixes() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const [$value, $setValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_","$"]}])),
            filename: None,
        },
    );
}

#[test]
fn invalid_with_ignoreprefixes_wrong_setter_after_stripping() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const [value, _wrong] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setValue" with a "set" prefix but got "_wrong"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [value, setValue] = useState('');");
}

#[test]
fn invalid_with_ignoreprefixes_wrong_setter_with_prefix_in_value() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const [_value, updateValue] = useState('');
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setValue" with a "set" prefix but got "updateValue"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [_value, setValue] = useState('');");
}

#[test]
fn invalid_with_ignoreprefixes_should_preserve_prefix_in_suggestion() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const [_data, wrongSetter] = useState([]);
        "#},
            options: Some(json!([{"ignorePrefixes":["_"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setData" with a "set" prefix but got "wrongSetter"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [_data, setData] = useState([]);");
}

#[test]
fn works_without_ignoreprefixes_option() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const [_value, _wrongSetter] = useState('');
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"

    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "set_value" with a "set" prefix but got "_wrongSetter"'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @"const [_value, set_value] = useState('');");
}

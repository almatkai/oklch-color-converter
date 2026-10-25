use serde_json::{json, Value};
use tezbar_oklch_color_converter::{convert, FORMATS};
#[test]
fn actual_pinned_own_tsx_and_culori_rendered_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../fixtures/oracle/results.json")).unwrap();
    let deviations: Value =
        serde_json::from_str(include_str!("../fixtures/oracle/numeric-deviations.json")).unwrap();
    let mut actual_deviations = vec![];
    for c in &cases {
        let input = c["input"].as_str().unwrap();
        let actual = serde_json::to_value(convert(input).unwrap()).unwrap();
        let expected = c["data"].as_array().unwrap();
        let actual = actual.as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{input}");
        let mut diffs = vec![];
        for (expected, actual) in expected.iter().zip(actual) {
            for (k, v) in expected.as_object().unwrap() {
                if &actual[k] != v {
                    diffs.push(json!([expected["format"], k, v, actual[k]]));
                }
            }
        }
        if !diffs.is_empty() {
            actual_deviations.push(json!([input, diffs]));
        }
        let rows = c["rendered"]["children"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s.get("children"))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        assert_eq!(
            rows.len(),
            expected.len(),
            "actual TSX rendered row count: {input}"
        );
        for (row, data) in rows.iter().zip(expected) {
            assert_eq!(row["props"]["title"], data["value"]);
            assert_eq!(row["props"]["subtitle"], data["format"]);
            assert_eq!(row["props"]["icon"]["source"], "Icon.CircleFilled");
            assert_eq!(row["props"]["icon"]["tintColor"], data["hexValue"]);
            assert_eq!(row["props"]["accessories"][0]["text"], data["tagText"]);
            let actions = &row["actions"]["children"][0]["children"];
            assert_eq!(actions[0]["type"], "Action.CopyToClipboard");
            assert_eq!(actions[0]["props"]["title"], "Copy Color Value");
            assert_eq!(actions[0]["props"]["content"], data["value"]);
            assert_eq!(
                actions[0]["props"]["shortcut"],
                json!({"modifiers":["cmd"],"key":"c"})
            );
            assert_eq!(actions[1]["props"]["title"], "Copy as Hex");
            assert_eq!(actions[1]["props"]["content"], data["hexValue"]);
            assert_eq!(
                actions[1]["props"]["shortcut"],
                json!({"modifiers":["cmd","shift"],"key":"c"})
            );
        }
    }
    assert_eq!(
        Value::Array(actual_deviations),
        deviations["cases"],
        "Unexpected new numeric drift or a resolved exception that should be removed"
    );
    assert_eq!(cases.len(), 897);
}
#[test]
fn independently_check_quirks_alpha_order_and_limits() {
    let p3 = convert("#ff000080").unwrap();
    let srgb = convert("ff000080").unwrap();
    assert_ne!(p3[1].value, srgb[1].value);
    assert_eq!(p3[0].value, "#FF000080");
    assert_eq!(
        p3.iter().map(|x| x.format.as_str()).collect::<Vec<_>>(),
        FORMATS
    );
    assert_eq!(convert("Figma P3 garbage").unwrap()[0].value, "#000000FF");
    assert_eq!(
        convert("color(display-p3 1 .502 0 / 0)").unwrap()[0].value,
        "#FF8000FF"
    );
    assert_eq!(convert("rgb(255 0 0 / .5)").unwrap()[5].value, "#FF0000");
    assert_eq!(
        convert("rgb(128 128 128 / .5)").unwrap()[4].value,
        "vec(0.03826, 0.03826, 0.03826, 0.50000)"
    );
    assert!(convert("vec(0, 0, 0, 1)").unwrap().is_empty());
    assert!(convert(" red").unwrap().is_empty());
    assert!(convert("rgb(1 2 3").unwrap().len() == 8);
    assert!(convert(&"x".repeat(65_536)).unwrap().is_empty());
    assert!(convert(&"x".repeat(65_537)).is_err());
    for input in ["rgb(1e999 0 0)", "rgb(0 0 0 / -1e999)", "oklch(.5 1e999 0)"] {
        assert_eq!(
            convert(input).unwrap_err(),
            "Color input contains a non-finite number."
        );
    }
    let c = tezbar_oklch_color_converter::parse("#1234").unwrap();
    assert_eq!(
        tezbar_oklch_color_converter::format_color(c, "hex/rgba")
            .unwrap()
            .value,
        "#11223344"
    );
    assert_eq!(
        tezbar_oklch_color_converter::format_color(c, "lrgb")
            .unwrap()
            .value,
        tezbar_oklch_color_converter::format_color(c, "vec")
            .unwrap()
            .value
    );
}

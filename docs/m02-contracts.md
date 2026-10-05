# M02 cursor and name contracts

`Cursor::new` validates the complete byte slice as UTF-8 and reports the first invalid byte offset. The cursor borrows its input; `remaining` and parsed names borrow the same input. Byte offsets are zero-based, while line and Unicode-scalar columns are one-based. CRLF counts as one newline, including when `advance` calls split the pair.

`peek` and `consume` reject XML 1.0 characters outside the supported character ranges. `advance` accepts a byte count only when it ends on a UTF-8 scalar boundary, validates each consumed character, and leaves the cursor unchanged on error. EOF is represented by `None` for peek and consume.

`parse_name` implements the XML 1.0 Fifth Edition `Name` character ranges, including colon at any valid name position. It consumes the maximal valid name and leaves the delimiter for its caller. Namespace prefix rules and QName validation are outside this function's contract.

All operations are allocation-free and safe for `no_std`; their running time is linear in the bytes inspected or consumed. Cursor and name errors carry stable offsets rather than allocated messages.

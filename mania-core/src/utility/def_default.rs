#[macro_export]
macro_rules! dda {
    ($struct:ident { $($field:ident $( : $value:expr )? ),* $(,)? }) => {
        $struct {
            $(
                $field: dda!(@value $field $( : $value )? )
            ),*,
            ..Default::default()
        }
    };

    (@value $field:ident : $value:expr) => {
        $value
    };

    (@value $field:ident) => {
        $field
    };
}

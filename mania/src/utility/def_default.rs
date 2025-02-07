#[macro_export]
macro_rules! dda {
    ($struct:ident { $($field:ident $( : $value:expr_2021 )? ),* $(,)? }) => {
        $struct {
            $(
                $field: dda!(@value $field $( : $value )? )
            ),*,
            ..Default::default()
        }
    };

    (@value $field:ident : $value:expr_2021) => {
        $value
    };

    (@value $field:ident) => {
        $field
    };
}

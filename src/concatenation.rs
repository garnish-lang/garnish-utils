use garnish_lang_traits::{GarnishData, GarnishDataType};

pub fn iterate_concatentation<
    Data: GarnishData,
    ItemFn: FnMut(Data::Size)
>(
    concat_index: Data::Size,
    data: &Data,
    mut item_fn: ItemFn
) -> Result<(), Data::Error> {
    let (current, next) = data.get_concatenation(concat_index)?;
    let mut stack = vec![];

    stack.push(next);
    stack.push(current);

    while let Some(addr) = stack.pop() {
        match data.get_data_type(addr)? {
            GarnishDataType::Concatenation => {
                let (current, next) = data.get_concatenation(addr)?;
                stack.push(next);
                stack.push(current);
            }
            GarnishDataType::List => {
                let list_iter = data.get_list_items_iter(addr);

                for i in list_iter {
                    let item = data.get_list_item(addr, i)?;

                    item_fn(item);
                }
            }
            _ => item_fn(addr),
        }
    }

    Ok(())
}

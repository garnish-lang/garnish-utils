use garnish_lang_traits::{GarnishData, GarnishDataType};

pub fn copy_data_at_to_data<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
) -> Result<Data::Size, Data::Error> {
    Ok(match from.get_data_type(data_addr)? {
        GarnishDataType::Invalid => todo!(),
        GarnishDataType::Unit => todo!(),
        GarnishDataType::Number => to.add_number(from.get_number(data_addr)?)?,
        GarnishDataType::Type => todo!(),
        GarnishDataType::Char => todo!(),
        GarnishDataType::CharList => todo!(),
        GarnishDataType::Byte => todo!(),
        GarnishDataType::ByteList => todo!(),
        GarnishDataType::Symbol => todo!(),
        GarnishDataType::Pair => todo!(),
        GarnishDataType::Range => todo!(),
        GarnishDataType::Concatenation => todo!(),
        GarnishDataType::Slice => todo!(),
        GarnishDataType::List => todo!(),
        GarnishDataType::Expression => todo!(),
        GarnishDataType::External => todo!(),
        GarnishDataType::True => todo!(),
        GarnishDataType::False => todo!(),
        GarnishDataType::Custom => todo!(),
    })
}

#[cfg(test)]
mod tests {
    use crate::data::copy_data_at_to_data;
    use garnish_lang_simple_data::{SimpleData, SimpleGarnishData, SimpleNumber};
    use garnish_lang_traits::GarnishData;

    #[test]
    fn copy_number() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_number(SimpleNumber::Integer(40)).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(
            to.get_data().get(6).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(40)
        );
    }
}

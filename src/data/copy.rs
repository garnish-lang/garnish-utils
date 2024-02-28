use garnish_lang_traits::{GarnishData, GarnishDataType};

pub fn copy_data_at_to_data<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
) -> Result<(), Data::Error> {
    match from.get_data_type(data_addr)? {
        GarnishDataType::Number => to.add_number(from.get_number(data_addr)?)?,
        _ => todo!(),
    };

    Ok(())
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

        copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(
            to.get_data().get(6).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(40)
        );
    }
}

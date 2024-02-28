use garnish_lang_traits::{GarnishData, GarnishDataType, TypeConstants};

pub fn copy_data_at_to_data<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
) -> Result<Data::Size, Data::Error> {
    match from.get_data_type(data_addr)? {
        GarnishDataType::Invalid => {
            unimplemented!("GarnishDataType::Invalid not supported to copy between data objects.")
        }
        GarnishDataType::Custom => {
            unimplemented!("GarnishDataType::Custom not supported to copy between data objects.")
        }
        GarnishDataType::Unit => to.add_unit(),
        GarnishDataType::Number => to.add_number(from.get_number(data_addr)?),
        GarnishDataType::Type => to.add_type(from.get_type(data_addr)?),
        GarnishDataType::Char => to.add_char(from.get_char(data_addr)?),
        GarnishDataType::CharList => {
            let len = from.get_char_list_len(data_addr)?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_char_list()?;
            for i in iter {
                to.add_to_char_list(from.get_char_list_item(data_addr, i)?)?;
            }

            to.end_char_list()
        }
        GarnishDataType::Byte => to.add_byte(from.get_byte(data_addr)?),
        GarnishDataType::ByteList => {
            let len = from.get_byte_list_len(data_addr)?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_byte_list()?;
            for i in iter {
                to.add_to_byte_list(from.get_byte_list_item(data_addr, i)?)?;
            }

            to.end_byte_list()
        }
        GarnishDataType::Symbol => to.add_symbol(from.get_symbol(data_addr)?),
        GarnishDataType::Pair => from.get_pair(data_addr)
            .and_then(|(left, right)| {
                let to_left = copy_data_at_to_data(left, from , to)?;
                let to_right = copy_data_at_to_data(right, from, to)?;
                to.add_pair((to_left, to_right))
            }),
        GarnishDataType::Range => from
            .get_range(data_addr)
            .and_then(|(s, e)| to.add_range(s, e)),
        GarnishDataType::Concatenation => todo!(),
        GarnishDataType::Slice => todo!(),
        GarnishDataType::List => todo!(),
        GarnishDataType::Expression => todo!(),
        GarnishDataType::External => to.add_external(from.get_external(data_addr)?),
        GarnishDataType::True => to.add_true(),
        GarnishDataType::False => to.add_false(),
    }
}

#[cfg(test)]
mod tests {
    use crate::data::copy_data_at_to_data;
    use garnish_lang_simple_data::{SimpleGarnishData, SimpleNumber};
    use garnish_lang_traits::{GarnishData, GarnishDataType};

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

    #[test]
    fn copy_unit() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_unit().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 0);
        assert!(to.get_data().get(0).unwrap().is_unit());
    }

    #[test]
    fn copy_true() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_true().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 2);
        assert!(to.get_data().get(2).unwrap().is_true());
    }

    #[test]
    fn copy_false() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_false().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 1);
        assert!(to.get_data().get(1).unwrap().is_false());
    }

    #[test]
    fn copy_type() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_type(GarnishDataType::Byte).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(
            to.get_data().get(6).unwrap().as_type().unwrap(),
            GarnishDataType::Byte
        );
    }

    #[test]
    fn copy_char() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_char('a').unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_char().unwrap(), 'a');
    }

    #[test]
    fn copy_char_list() {
        let mut from = SimpleGarnishData::new();
        let addr = from.parse_add_char_list("\"stuff\"").unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(
            to.get_data().get(6).unwrap().as_char_list().unwrap(),
            "stuff"
        );
    }

    #[test]
    fn copy_byte() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_byte(10).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_byte().unwrap(), 10);
    }

    #[test]
    fn copy_byte_list() {
        let mut from = SimpleGarnishData::new();
        let addr = from.parse_add_byte_list("''100 150 200''").unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(
            to.get_data().get(6).unwrap().as_byte_list().unwrap(),
            vec![100, 150, 200]
        );
    }

    #[test]
    fn copy_range() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_range(10, 20).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_range().unwrap(), (10, 20));
    }

    #[test]
    fn copy_symbol() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_symbol(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
    }

    #[test]
    fn copy_external() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_external(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_external().unwrap(), 100);
    }

    #[test]
    fn copy_pair() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_symbol(100).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(200)).unwrap();
        let d3 = from.add_pair((d1, d2)).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = copy_data_at_to_data(d3, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_pair().unwrap(), (6, 7));
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
    }
}

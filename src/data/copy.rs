use garnish_lang_traits::{GarnishData, GarnishDataType, TypeConstants};

pub type CloneHandler<Data> = fn(<Data as GarnishData>::Size, &Data, &mut Data) -> Result<<Data as GarnishData>::Size, <Data as GarnishData>::Error>;

pub fn clone_data<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers(
        data_addr,
        from,
        to,
        None,
    )
}

pub fn clone_data_with_custom_handler<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers(
        data_addr,
        from,
        to,
        Some(custom_handler),
    )
}

pub fn clone_data_with_handlers<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: Option<CloneHandler<Data>>,
    // invalid_handler: Option<CloneHandler<Data>>, // to be implemented
) -> Result<Data::Size, Data::Error> {
    match from.get_data_type(data_addr.clone())? {
        GarnishDataType::Invalid => to.add_unit(), // will add custom handler
        GarnishDataType::Custom => match custom_handler {
            None => to.add_unit(),
            Some(handler) => handler(data_addr.clone(), from, to)
        }
        GarnishDataType::Unit => to.add_unit(),
        GarnishDataType::Number => to.add_number(from.get_number(data_addr.clone())?),
        GarnishDataType::Type => to.add_type(from.get_type(data_addr.clone())?),
        GarnishDataType::Char => to.add_char(from.get_char(data_addr.clone())?),
        GarnishDataType::CharList => {
            let len = from.get_char_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_char_list()?;
            for i in iter {
                to.add_to_char_list(from.get_char_list_item(data_addr.clone(), i)?)?;
            }

            to.end_char_list()
        }
        GarnishDataType::Byte => to.add_byte(from.get_byte(data_addr.clone())?),
        GarnishDataType::ByteList => {
            let len = from.get_byte_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_byte_list()?;
            for i in iter {
                to.add_to_byte_list(from.get_byte_list_item(data_addr.clone(), i)?)?;
            }

            to.end_byte_list()
        }
        GarnishDataType::Symbol => to.add_symbol(from.get_symbol(data_addr.clone())?),
        GarnishDataType::Pair => from.get_pair(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data(left, from, to)?;
            let to_right = clone_data(right, from, to)?;
            to.add_pair((to_left, to_right))
        }),
        GarnishDataType::Range => from.get_range(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data(left, from, to)?;
            let to_right = clone_data(right, from, to)?;
            to.add_range(to_left, to_right)
        }),
        GarnishDataType::Concatenation => {
            from.get_concatenation(data_addr.clone()).and_then(|(left, right)| {
                let to_left = clone_data(left, from, to)?;
                let to_right = clone_data(right, from, to)?;
                to.add_concatenation(to_left, to_right)
            })
        }
        GarnishDataType::Slice => from.get_slice(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data(left, from, to)?;
            let to_right = clone_data(right, from, to)?;
            to.add_slice(to_left, to_right)
        }),
        GarnishDataType::List => {
            let len = from.get_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len.clone()));
            to.start_list(len.clone())?;
            for i in iter {
                let addr = from
                    .get_list_item(data_addr.clone(), i)
                    .and_then(|addr| clone_data(addr, from, to))?;
                let is_association = match to.get_data_type(addr.clone())? {
                    GarnishDataType::Pair => {
                        let (left, _right) = to.get_pair(addr.clone())?;
                        match to.get_data_type(left)? {
                            GarnishDataType::Symbol => true,
                            _ => false
                        }
                    }
                    _ => false
                };
                to.add_to_list(addr, is_association)?;
            }

            to.end_list()
        }
        GarnishDataType::Expression => to.add_expression(from.get_expression(data_addr.clone())?),
        GarnishDataType::External => to.add_external(from.get_external(data_addr.clone())?),
        GarnishDataType::True => to.add_true(),
        GarnishDataType::False => to.add_false(),
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{clone_data, clone_data_with_custom_handler};
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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

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

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(
            to.get_data().get(6).unwrap().as_byte_list().unwrap(),
            vec![100, 150, 200]
        );
    }

    #[test]
    fn copy_range() {
        let mut from = SimpleGarnishData::new();
        let s = from.add_number(SimpleNumber::Integer(1)).unwrap();
        let e = from.add_number(SimpleNumber::Integer(5)).unwrap();
        let addr = from.add_range(s, e).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_range().unwrap(), (6, 7));
        assert_eq!(
            to.get_data().get(6).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(1)
        );
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(5)
        );
    }

    #[test]
    fn copy_symbol() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_symbol(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
    }

    #[test]
    fn copy_expression() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_expression(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_expression().unwrap(), 100);
    }

    #[test]
    fn copy_external() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_external(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_external().unwrap(), 100);
    }


    #[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
    struct CustomData {
        num: usize,
    }

    #[test]
    fn copy_custom_no_handler() {
        let mut from = SimpleGarnishData::<CustomData>::new_custom();
        let addr = from.add_custom(CustomData { num: 12345 }).unwrap();

        let mut to = SimpleGarnishData::<CustomData>::new_custom();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 0);
        assert!(to.get_data().get(0).unwrap().is_unit());
    }

    #[test]
    fn copy_custom_handler() {
        let mut from = SimpleGarnishData::<CustomData>::new_custom();
        let addr = from.add_custom(CustomData { num: 12345 }).unwrap();

        let mut to = SimpleGarnishData::<CustomData>::new_custom();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data_with_custom_handler(
            addr, &from, &mut to,
            |_addr, _from, to| {
                to.add_number(SimpleNumber::Integer(12345))
            }).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(12345));
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

        let new_addr = clone_data(d3, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_pair().unwrap(), (6, 7));
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(200)
        );
    }

    #[test]
    fn copy_concatenation() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_symbol(100).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(200)).unwrap();
        let d3 = from.add_concatenation(d1, d2).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d3, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(
            to.get_data().get(8).unwrap().as_concatenation().unwrap(),
            (6, 7)
        );
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(200)
        );
    }

    #[test]
    fn copy_list() {
        let mut from = SimpleGarnishData::new();
        from.start_list(3).unwrap();
        from.add_number(SimpleNumber::Integer(100))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(200))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(300))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        let d4 = from.end_list().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d4, &from, &mut to).unwrap();

        assert_eq!(new_addr, 9);
        assert_eq!(
            to.get_data().get(9).unwrap().as_list().unwrap(),
            (vec![6, 7, 8], vec![])
        );
        assert_eq!(
            to.get_data().get(6).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(100)
        );
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(200)
        );
        assert_eq!(
            to.get_data().get(8).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(300)
        );
    }

    #[test]
    fn copy_list_with_associations() {
        let mut from = SimpleGarnishData::new();
        from.start_list(3).unwrap();

        let left = from.add_symbol(200).unwrap();
        let right = from.add_number(SimpleNumber::Integer(100)).unwrap();
        from.add_pair((left, right))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        let d4 = from.end_list().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d4, &from, &mut to).unwrap();

        assert_eq!(new_addr, 9);
        assert_eq!(
            to.get_data().get(9).unwrap().as_list().unwrap(),
            (vec![8], vec![8])
        );
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(100)
        );
        assert_eq!(
            to.get_data().get(6).unwrap().as_symbol().unwrap(),
            200
        );
    }

    #[test]
    fn copy_slice() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_number(SimpleNumber::Integer(1)).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(3)).unwrap();
        let d3 = from.add_range(d1, d2).unwrap();
        from.start_list(3).unwrap();
        from.add_number(SimpleNumber::Integer(100))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(200))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(300))
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        let d4 = from.end_list().unwrap();
        let d5 = from.add_slice(d4, d3).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d5, &from, &mut to).unwrap();

        assert_eq!(new_addr, 13);
        assert_eq!(to.get_data().get(13).unwrap().as_slice().unwrap(), (9, 12));
        assert_eq!(to.get_data().get(12).unwrap().as_range().unwrap(), (10, 11));
        assert_eq!(
            to.get_data().get(10).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(1)
        );
        assert_eq!(
            to.get_data().get(11).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(3)
        );
        assert_eq!(
            to.get_data().get(9).unwrap().as_list().unwrap(),
            (vec![6, 7, 8], vec![])
        );
        assert_eq!(
            to.get_data().get(6).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(100)
        );
        assert_eq!(
            to.get_data().get(7).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(200)
        );
        assert_eq!(
            to.get_data().get(8).unwrap().as_number().unwrap(),
            SimpleNumber::Integer(300)
        );
    }
}

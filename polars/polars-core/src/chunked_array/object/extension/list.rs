use crate::chunked_array::object::builder::ObjectChunkedBuilder;
use crate::chunked_array::object::extension::create_extension;
use crate::prelude::*;

impl<T: PolarsObject> ObjectChunked<T> {
    pub(crate) fn get_list_builder(
        name: &str,
        values_capacity: usize,
        list_capacity: usize,
    ) -> Box<dyn ListBuilderTrait> {
        Box::new(ExtensionListBuilder::<T>::new(
            name,
            values_capacity,
            list_capacity,
        ))
    }
}

struct ExtensionListBuilder<T: PolarsObject> {
    values_builder: ObjectChunkedBuilder<T>,
    offsets: Vec<i64>,
    fast_explode: bool,
}

impl<T: PolarsObject> ExtensionListBuilder<T> {
    pub(crate) fn new(name: &str, values_capacity: usize, list_capacity: usize) -> Self {
        let mut offsets = Vec::with_capacity(list_capacity + 1);
        offsets.push(0);
        Self {
            values_builder: ObjectChunkedBuilder::new(name, values_capacity),
            offsets,
            fast_explode: true,
        }
    }
}

impl<T: PolarsObject> ListBuilderTrait for ExtensionListBuilder<T> {
    fn append_series(&mut self, s: &Series) {
        let arr = s
            .as_any()
            .downcast_ref::<ObjectChunked<T>>()
            .expect("series of type object");

        for v in arr.into_iter() {
            self.values_builder.append_option(v.cloned())
        }
        if arr.is_empty() {
            self.fast_explode = false;
        }
        let len_so_far = self.offsets[self.offsets.len() - 1];
        self.offsets.push(len_so_far + arr.len() as i64);
    }

    fn append_null(&mut self) {
        self.values_builder.append_null();
        let len_so_far = self.offsets[self.offsets.len() - 1];
        self.offsets.push(len_so_far + 1);
    }

    fn finish(&mut self) -> ListChunked {
        let values_builder = std::mem::take(&mut self.values_builder);
        let offsets = std::mem::take(&mut self.offsets);
        let ca = values_builder.finish();
        let obj_arr = ca.downcast_chunks().get(0).unwrap().clone();

        let mut pe = create_extension(obj_arr.into_iter_cloned());

        // Safety:
        // this is safe because we just created the PolarsExtension
        // meaning that the sentinel is heap allocated and the dereference of the
        // pointer does not fail
        unsafe { pe.set_to_series_fn::<T>() };
        let extension_array = Arc::new(pe.take_and_forget()) as ArrayRef;
        let extension_dtype = extension_array.data_type();

        let data_type = ListArray::<i64>::default_datatype(extension_dtype.clone());
        let arr = Arc::new(ListArray::<i64>::from_data(
            data_type,
            offsets.into(),
            extension_array,
            None,
        )) as ArrayRef;

        let mut listarr = ListChunked::new_from_chunks(ca.name(), vec![arr]);
        if self.fast_explode {
            listarr.set_fast_explode()
        }
        listarr
    }
}

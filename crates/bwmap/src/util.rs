use std::mem::MaybeUninit;
use tracing::instrument;

#[instrument(level = "trace", skip_all)]
pub(crate) fn parse_slice<T: Copy>(s: &[u8]) -> T {
    if std::mem::size_of::<T>() != s.len() {
        panic!(
            "sizeof<t>: {}, s.len(): {}",
            std::mem::size_of::<T>(),
            s.len()
        );
    }

    unsafe { *(s.as_ptr() as *const T) }
}

// pub(crate) fn parse_slice2<T: Copy>(s: &mut std::io::Cursor<&[u8]>) -> Result<T, anyhow::Error> {
//     let mut buffer = Vec::new();

//     buffer.resize(std::mem::size_of::<T>(), 0); // TODO: could do this at compile time when rust suppports generic const expressions

//     s.read_exact(buffer.as_mut_slice())?;

//     Ok(unsafe { *(buffer.as_ptr() as *const T) })
// }

// pub(crate) fn parse_slice_to_vec<T: Copy>(
//     s: &mut std::io::Cursor<&[u8]>,
//     count: usize,
// ) -> Result<Vec<T>, anyhow::Error> {
//     let mut buffer = Vec::new();

//     buffer.resize(count * std::mem::size_of::<T>(), 0);

//     s.read(buffer.as_mut_slice())?;

//     unsafe {
//         Ok(Vec::from(std::slice::from_raw_parts(
//             buffer.as_ptr() as *const T,
//             buffer.len() / std::mem::size_of::<T>(),
//         )))
//     }
// }

#[instrument(level = "trace", skip_all)]
#[cfg(feature = "full")]
pub(crate) fn reinterpret_as_slice<T: Sized + Copy>(s: &T) -> Result<&[u8], anyhow::Error> {
    anyhow::Ok(unsafe {
        std::slice::from_raw_parts((s as *const T) as *const u8, std::mem::size_of::<T>())
    })
}

#[instrument(level = "trace", skip_all)]
pub(crate) fn reinterpret_slice2<T: Sized>(s: &[u8]) -> Result<&[T], anyhow::Error> {
    anyhow::ensure!(
        s.len() % std::mem::size_of::<T>() == 0,
        "s.len(): {}, std::mem::size_of::<T>(): {}",
        s.len(),
        std::mem::size_of::<T>()
    );

    Ok(unsafe {
        std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / std::mem::size_of::<T>())
    })
}

#[instrument(level = "trace", skip_all)]
pub(crate) fn reinterpret_slice3<T: Copy + Sized>(s: &[u8]) -> Result<T, anyhow::Error> {
    anyhow::ensure!(
        s.len() == std::mem::size_of::<T>(),
        "s.len(): {}, std::mem::size_of::<T>(): {}",
        s.len(),
        std::mem::size_of::<T>()
    );

    Ok(unsafe { std::ptr::read_unaligned(s.as_ptr() as *const T) })
}

trait Extract: Sized + Copy {
    fn extract(data: &[u8]) -> anyhow::Result<(Self, &[u8])>;
}

trait ExtractLaxZeroPad: Sized + Copy {
    fn extract_lax_zero_pad(data: &[u8]) -> (Self, &[u8]);
}

impl<T: Extract> ExtractLaxZeroPad for T {
    fn extract_lax_zero_pad(data: &[u8]) -> (Self, &[u8]) {
        // I suppose it's not technically correct that if the data length is std::mem::size_of::<T>() then you can extract a T out of it.
        // Probably for at least two reasons, it might be that the representation is illegal, or it might be that the sizes don't match.
        // Because there is only a convenient matching between the rust types and what need to be extracted from the data.
        // TODO: there should be a way how to do this without dynamic memory allocation but I can't figure out the right rust way.
        if data.len() >= std::mem::size_of::<Self>() {
            return T::extract(data).unwrap();
        }

        // Need to zero pad.
        let mut data2: Vec<u8> = vec![0u8; std::mem::size_of::<Self>()];
        data2[0..data.len()].copy_from_slice(data);

        (T::extract(&data2).unwrap().0, &[])
    }
}

impl Extract for u8 {
    fn extract(data: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        Ok((
            Self::from_le_bytes(data[0..std::mem::size_of::<Self>()].try_into()?),
            &data[std::mem::size_of::<Self>()..],
        ))
    }
}

impl Extract for u16 {
    fn extract(data: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        Ok((
            Self::from_le_bytes(data[0..std::mem::size_of::<Self>()].try_into()?),
            &data[std::mem::size_of::<Self>()..],
        ))
    }
}

impl Extract for u32 {
    fn extract(data: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        Ok((
            Self::from_le_bytes(data[0..std::mem::size_of::<Self>()].try_into()?),
            &data[std::mem::size_of::<Self>()..],
        ))
    }
}

impl<T: Extract, const N: usize> Extract for [T; N] {
    fn extract(data: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        let mut ret: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut remainder = data;
        for i in 0..N {
            let (v, r) = T::extract(remainder)?;
            remainder = r;
            ret[i].write(v);
        }

        // TODO: change this to std::mem::transmute when this issue is resolved:
        // https://github.com/rust-lang/rust/issues/61956
        let ret2 = unsafe { std::mem::transmute_copy::<_, [T; N]>(&ret) };
        #[allow(forgetting_copy_types)]
        std::mem::forget(ret);

        Ok((ret2, remainder))
    }
}

pub(crate) struct CursorSlicer<'a> {
    s: &'a [u8],
    current_offset: usize,
}

impl<'a> CursorSlicer<'a> {
    #[instrument(level = "trace", skip_all)]
    pub(crate) fn new(s: &'a [u8]) -> CursorSlicer<'a> {
        CursorSlicer {
            s,
            current_offset: 0,
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_slice<T>(&mut self, elements: usize) -> Result<&'a [T], anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset + elements * std::mem::size_of::<T>());

        let ret = reinterpret_slice2(
            &self.s[self.current_offset..self.current_offset + elements * std::mem::size_of::<T>()],
        )?;

        self.current_offset += std::mem::size_of_val(ret);

        Ok(ret)
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_slice_lax<T>(
        &mut self,
        elements: usize,
    ) -> Result<&'a [T], anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset);

        let elements = std::cmp::min(
            (self.s.len() - self.current_offset) / std::mem::size_of::<T>(),
            elements,
        );

        let ret = reinterpret_slice2(
            &self.s[self.current_offset..self.current_offset + elements * std::mem::size_of::<T>()],
        )?;

        self.current_offset += std::mem::size_of_val(ret);

        Ok(ret)
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_rest_as_slice<T>(&mut self) -> Result<&'a [T], anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset);
        anyhow::ensure!((self.s.len() - self.current_offset) % std::mem::size_of::<T>() == 0);

        let elements = (self.s.len() - self.current_offset) / std::mem::size_of::<T>();

        self.extract_slice(elements)
    }

    // If for example there is some kind of protection where one of the objects is mangled, such as [int, int, int, X] where X is 1 byte instead of 4, the lax variant will ignore the last one.
    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_rest_as_slice_lax<T>(&mut self) -> Result<&'a [T], anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset);
        //anyhow::ensure!((self.s.len() - self.current_offset) % std::mem::size_of::<T>() == 0);

        let elements = (self.s.len() - self.current_offset) / std::mem::size_of::<T>();

        self.extract_slice_lax(elements)
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_ref<T>(&mut self) -> Result<&'a T, anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset + std::mem::size_of::<T>());

        let ret = &reinterpret_slice2(
            &self.s[self.current_offset..self.current_offset + std::mem::size_of::<T>()],
        )?[0];
        self.current_offset += std::mem::size_of::<T>();
        Ok(ret)
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_ref_lax<T>(&mut self) -> Result<Option<&'a T>, anyhow::Error> {
        anyhow::ensure!(self.s.len() >= self.current_offset);

        if self.s.len() >= self.current_offset + std::mem::size_of::<T>() {
            Ok(Some(self.extract_ref()?))
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_u8_lax(&mut self) -> u8 {
        let ret = if self.s.len() > self.current_offset {
            let ret = self.s[self.current_offset];
            self.current_offset += 1;

            ret
        } else {
            0
        };

        return ret;
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_u16_lax(&mut self) -> u16 {
        if self.s.len() <= self.current_offset {
            let lower_16_bits = 0_u16;
            let upper_16_bits = 0_u16;
            self.current_offset += 0;
            upper_16_bits << 8 | lower_16_bits
        } else if self.s.len() <= self.current_offset + 1 {
            let lower_16_bits = self.s[self.current_offset] as u16;
            let upper_16_bits = 0_u16;
            self.current_offset += 1;
            upper_16_bits << 8 | lower_16_bits
        } else {
            let lower_16_bits = self.s[self.current_offset] as u16;
            let upper_16_bits = self.s[self.current_offset + 1] as u16;
            self.current_offset += 2;
            upper_16_bits << 8 | lower_16_bits
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_u8_array_lax<const N: usize>(&mut self) -> [u8; N] {
        let mut ret = [0u8; N];

        for i in 0..N {
            ret[i] = self.extract_u8_lax();
        }

        ret
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_u16_array_lax<const N: usize>(&mut self) -> [u16; N] {
        let mut ret = [0u16; N];

        for i in 0..N {
            ret[i] = self.extract_u16_lax();
        }

        ret
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract<T: Extract>(&mut self) -> anyhow::Result<T> {
        let (ret, s) = T::extract(self.s)?;
        self.s = s;
        Ok(ret)
    }

    // #[instrument(level = "trace", skip_all)]
    // pub(crate) fn extract_vec<T: Extract>(&mut self) -> anyhow::Result<Vec<T>> {
    //     let mut ret = Vec::new();

    //     loop {
    //         if self.s.len() == 0 {
    //             break;
    //         }

    //         let (v, s) = T::extract(self.s)?;
    //         ret.push(v);
    //         self.s = s;
    //     }

    //     ret
    // }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn extract_lax_zero_pad<T: Extract>(&mut self) -> T {
        let (ret, s) = T::extract_lax_zero_pad(self.s);
        self.s = s;
        ret
    }

    // #[instrument(level = "trace", skip_all)]
    // pub(crate) fn extract_array_lax<T: Extract, const N: usize>(&mut self) -> [T; N] {
    //     let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    //     for i in 0..N {
    //         data[i].write(self.extract_T());
    //     }

    //     // Everything is initialized. Transmute the array to the
    //     // initialized type.
    //     unsafe { std::mem::transmute::<_, [T; N]>(data) }
    // }
}

#[instrument(level = "trace", skip_all)]
#[cfg(feature = "full")]
pub(crate) fn parse_null_terminated_bytestring_unsigned(s: &[u8]) -> &[u8] {
    let mut index = 0;
    for &c in s {
        if c == 0 {
            break;
        }

        index += 1;
    }

    &s[0..index]
}

// pub(crate) fn reinterpret_slice2<T: Copy + Sized>(s: &[i8]) -> &[T] {
//     if s.len() % std::mem::size_of::<T>() != 0 {
//         panic!();
//     }

//     unsafe {
//         std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / std::mem::size_of::<T>())
//     }
// }

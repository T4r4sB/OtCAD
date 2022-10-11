use std::marker::PhantomData;
use std::ops::{Bound::*, Index, IndexMut, RangeBounds};
use std::slice::{from_raw_parts, from_raw_parts_mut};

macro_rules! lines {
    ($self: ident, $range: ident, $result: ident) => {
        unsafe {
            let first = match $range.start_bound() {
                Included(first) => *first,
                Excluded(first) => *first + 1,
                Unbounded => 0,
            };

            let last = match $range.end_bound() {
                Included(last) => *last + 1,
                Excluded(last) => *last,
                Unbounded => $self.size.1,
            };

            assert!(last <= $self.size.1);

            return $result::from_raw(
                $self.memory.add(first * $self.stride),
                $self.memory.add(last * $self.stride),
                $self.size.0,
                $self.stride,
            );
        }
    };
}

macro_rules! window {
    ($self: expr, $left_top: ident, $right_bottom: ident, $result: ident) => {
        assert!($left_top.0 <= $right_bottom.0);
        assert!($right_bottom.0 <= $self.size.0);
        assert!($left_top.1 <= $right_bottom.1);
        assert!($right_bottom.1 <= $self.size.1);
        unsafe {
            return $result::from_raw(
                $self.memory.add($left_top.1 * $self.stride + $left_top.0),
                ($right_bottom.0 - $left_top.0, $right_bottom.1 - $left_top.1),
                $self.stride,
            );
        }
    };
}

macro_rules! index {
    ($self: ident, $y: ident, $from_raw_parts: ident) => {
        assert!($y < $self.size.1);
        unsafe {
            return $from_raw_parts($self.memory.add($y * $self.stride), $self.size.0);
        }
    };
}

pub type ImageSize = (usize, usize);
pub type Position = (i32, i32);

pub fn position_to_image_size(position: Position) -> ImageSize {
    (position.0 as usize, position.1 as usize)
}

pub fn image_size_to_position(image_size: ImageSize) -> Position {
    (image_size.0 as i32, image_size.1 as i32)
}

pub struct ImageViewMut<'i, Pixel> {
    memory: *mut Pixel,
    size: ImageSize,
    stride: usize,
    lifetime_marker: PhantomData<&'i Pixel>,
}

impl<'i, Pixel> ImageViewMut<'i, Pixel> {
    pub unsafe fn from_raw(memory: *mut Pixel, size: ImageSize, stride: usize) -> Self {
        Self {
            memory,
            size,
            stride,
            lifetime_marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const Pixel {
        self.memory
    }

    pub fn as_mut_ptr(&mut self) -> *mut Pixel {
        self.memory
    }

    pub fn get_size(&self) -> ImageSize {
        self.size
    }

    pub fn lines(&self, range: impl RangeBounds<usize>) -> LineIter<'i, Pixel> {
        lines!(self, range, LineIter);
    }

    pub fn window(&self, left_top: ImageSize, right_bottom: ImageSize) -> ImageView<Pixel> {
        window!(self, left_top, right_bottom, ImageView);
    }

    pub fn lines_mut(&mut self, range: impl RangeBounds<usize>) -> LineIterMut<'i, Pixel> {
        lines!(self, range, LineIterMut);
    }

    pub fn window_mut(
        &mut self,
        left_top: ImageSize,
        right_bottom: ImageSize,
    ) -> ImageViewMut<Pixel> {
        window!(self, left_top, right_bottom, ImageViewMut);
    }

    pub fn as_view(&self) -> ImageView<Pixel> {
        unsafe {
            return ImageView::from_raw(self.memory, self.size, self.stride);
        }
    }

    pub fn draw<SrcPixel>(
        &mut self,
        src: &ImageView<SrcPixel>,
        position: Position,
        apply: impl Fn(&mut Pixel, &SrcPixel),
    ) {
        if position.0 >= self.size.0 as i32
            || position.1 >= self.size.1 as i32
            || position.0 + src.size.0 as i32 <= 0
            || position.1 + src.size.1 as i32 <= 0
        {
            return;
        }

        let left_top_x = if position.0 < 0 {
            (0, (-position.0) as usize)
        } else {
            (position.0 as usize, 0)
        };

        let left_top_y = if position.1 < 0 {
            (0, (-position.1) as usize)
        } else {
            (position.1 as usize, 0)
        };

        let right_bottom_x = if position.0 + src.size.0 as i32 > self.size.0 as i32 {
            (self.size.0, (self.size.0 as i32 - position.0) as usize)
        } else {
            ((position.0 + src.size.0 as i32) as usize, src.size.0)
        };

        let right_bottom_y = if position.1 + src.size.1 as i32 > self.size.1 as i32 {
            (self.size.1, (self.size.1 as i32 - position.1) as usize)
        } else {
            ((position.1 + src.size.1 as i32) as usize, src.size.1)
        };

        self.window_mut(
            (left_top_x.0, left_top_y.0),
            (right_bottom_x.0, right_bottom_y.0),
        )
        .draw_same_size(
            src.window(
                (left_top_x.1, left_top_y.1),
                (right_bottom_x.1, right_bottom_y.1),
            ),
            apply,
        );
    }

    fn draw_same_size<SrcPixel>(
        &mut self,
        src: ImageView<SrcPixel>,
        apply: impl Fn(&mut Pixel, &SrcPixel),
    ) {
        assert!(self.size == src.size);
        for (dst_line, src_line) in self.lines_mut(..).zip(src.lines(..)) {
            for (dst, src) in dst_line.iter_mut().zip(src_line.iter()) {
                apply(dst, src);
            }
        }
    }

    pub fn fill(&mut self, apply: impl Fn(&mut Pixel)) {
        unsafe {
            let mut dsty = self.memory;
            let dsty_end = self.memory.add(self.stride * self.size.1);
            while dsty < dsty_end {
                let mut dstx = dsty;
                let dstx_end = dstx.add(self.size.0);
                while dstx < dstx_end {
                    apply(&mut *dstx);
                    dstx = dstx.add(1);
                }
                dsty = dsty.add(self.stride);
            }
        }
    }

    pub fn fill_with_coord(&mut self, apply: impl Fn(&mut Pixel, (usize, usize))) {
        unsafe {
            let mut y = 0;
            let mut dsty = self.memory;
            let dsty_end = self.memory.add(self.stride * self.size.1);
            while dsty < dsty_end {
                let mut x = 0;
                let mut dstx = dsty;
                let dstx_end = dstx.add(self.size.0);
                while dstx < dstx_end {
                    apply(&mut *dstx, (x, y));
                    x += 1;
                    dstx = dstx.add(1);
                }
                y += 1;
                dsty = dsty.add(self.stride);
            }
        }
    }
}

pub struct ImageView<'i, Pixel> {
    memory: *const Pixel,
    size: ImageSize,
    stride: usize,
    lifetime_marker: PhantomData<&'i Pixel>,
}

impl<'i, Pixel> ImageView<'i, Pixel> {
    pub unsafe fn from_raw(memory: *const Pixel, size: ImageSize, stride: usize) -> Self {
        Self {
            memory,
            size,
            stride,
            lifetime_marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const Pixel {
        self.memory
    }

    pub fn get_size(&self) -> ImageSize {
        self.size
    }

    pub fn lines(&self, range: impl RangeBounds<usize>) -> LineIter<'i, Pixel> {
        lines!(self, range, LineIter);
    }

    pub fn window(&self, left_top: ImageSize, right_bottom: ImageSize) -> ImageView<Pixel> {
        window!(self, left_top, right_bottom, ImageView);
    }
}

impl<'i, Pixel> Index<usize> for ImageView<'i, Pixel> {
    type Output = [Pixel];
    fn index(&self, y: usize) -> &[Pixel] {
        index!(self, y, from_raw_parts);
    }
}

impl<'i, Pixel> Index<usize> for ImageViewMut<'i, Pixel> {
    type Output = [Pixel];
    fn index(&self, y: usize) -> &[Pixel] {
        index!(self, y, from_raw_parts);
    }
}

impl<'i, Pixel> IndexMut<usize> for ImageViewMut<'i, Pixel> {
    fn index_mut(&mut self, y: usize) -> &mut [Pixel] {
        index!(self, y, from_raw_parts_mut);
    }
}

pub struct LineIter<'i, Pixel> {
    first: *const Pixel,
    last: *const Pixel,
    size_x: usize,
    stride: usize,
    lifetime_marker: PhantomData<&'i Pixel>,
}

impl<'i, Pixel> LineIter<'i, Pixel> {
    unsafe fn from_raw(
        first: *const Pixel,
        last: *const Pixel,
        size_x: usize,
        stride: usize,
    ) -> Self {
        Self {
            first,
            last,
            size_x,
            stride,
            lifetime_marker: PhantomData,
        }
    }
}

impl<'i, Pixel: 'i> Iterator for LineIter<'i, Pixel> {
    type Item = &'i [Pixel];
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.first.add(self.size_x) > self.last {
                None
            } else {
                let result = from_raw_parts(self.first, self.size_x);
                self.first = self.first.add(self.stride);
                Some(result)
            }
        }
    }
}

pub struct LineIterMut<'i, Pixel> {
    first: *mut Pixel,
    last: *mut Pixel,
    size_x: usize,
    stride: usize,
    lifetime_marker: PhantomData<&'i Pixel>,
}

impl<'i, Pixel> LineIterMut<'i, Pixel> {
    unsafe fn from_raw(first: *mut Pixel, last: *mut Pixel, size_x: usize, stride: usize) -> Self {
        Self {
            first,
            last,
            size_x,
            stride,
            lifetime_marker: PhantomData,
        }
    }
}

impl<'i, Pixel: 'i> Iterator for LineIterMut<'i, Pixel> {
    type Item = &'i mut [Pixel];
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.first.add(self.size_x) > self.last {
                None
            } else {
                let result = from_raw_parts_mut(self.first, self.size_x);
                self.first = self.first.add(self.stride);
                Some(result)
            }
        }
    }
}

pub struct Image<Pixel: Default> {
    data: Vec<Pixel>,
    size: ImageSize,
}

impl<Pixel: Default> Image<Pixel> {
    pub fn new(size: ImageSize) -> Self {
        let mut data = Vec::new();
        data.resize_with(size.0 * size.1, || Pixel::default());
        Self { data, size }
    }

    pub fn get_size(&self) -> ImageSize {
        self.size
    }

    pub fn get_data(&self) -> &[Pixel] {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut [Pixel] {
        &mut self.data
    }

    pub fn as_view(&self) -> ImageView<Pixel> {
        unsafe { ImageView::from_raw(self.data.as_ptr(), self.size, self.size.0) }
    }

    pub fn as_view_mut(&mut self) -> ImageViewMut<Pixel> {
        unsafe { ImageViewMut::from_raw(self.data.as_mut_ptr(), self.size, self.size.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_two_images() -> (Image<u8>, Image<u8>) {
        let mut src = Image::<u8>::new((4, 4));
        src.as_view_mut().fill(|d| *d = 42);
        let mut dst = Image::<u8>::new((4, 4));
        dst.as_view_mut().fill(|d| *d = 17);
        (src, dst)
    }

    #[test]
    fn test_draw_same_size() {
        let (src, mut dst) = init_two_images();
        dst.as_view_mut()
            .draw(src.as_view(), (0, 0), |d, s| *d = *s);
        assert_eq!(dst.as_view()[0][0], 42);
        assert_eq!(dst.as_view()[0][3], 42);
        assert_eq!(dst.as_view()[3][0], 42);
        assert_eq!(dst.as_view()[3][3], 42);
    }

    #[test]
    fn test_draw_same_size_shifted_neg_neg() {
        let (src, mut dst) = init_two_images();
        dst.as_view_mut()
            .draw(src.as_view(), (-2, -2), |d, s| *d = *s);
        assert_eq!(dst.as_view()[0][0], 42);
        assert_eq!(dst.as_view()[0][3], 17);
        assert_eq!(dst.as_view()[3][0], 17);
        assert_eq!(dst.as_view()[3][3], 17);
    }

    #[test]
    fn test_draw_same_size_shifted_neg_pos() {
        let (src, mut dst) = init_two_images();
        dst.as_view_mut()
            .draw(src.as_view(), (-2, 2), |d, s| *d = *s);
        assert_eq!(dst.as_view()[0][0], 17);
        assert_eq!(dst.as_view()[0][3], 17);
        assert_eq!(dst.as_view()[3][0], 42);
        assert_eq!(dst.as_view()[3][3], 17);
    }

    #[test]
    fn test_draw_same_size_shifted_pos_neg() {
        let (src, mut dst) = init_two_images();
        dst.as_view_mut()
            .draw(src.as_view(), (2, -2), |d, s| *d = *s);
        assert_eq!(dst.as_view()[0][0], 17);
        assert_eq!(dst.as_view()[0][3], 42);
        assert_eq!(dst.as_view()[3][0], 17);
        assert_eq!(dst.as_view()[3][3], 17);
    }

    #[test]
    fn test_draw_same_size_shifted_pos_pos() {
        let (src, mut dst) = init_two_images();
        dst.as_view_mut()
            .draw(src.as_view(), (2, 2), |d, s| *d = *s);
        assert_eq!(dst.as_view()[0][0], 17);
        assert_eq!(dst.as_view()[0][3], 17);
        assert_eq!(dst.as_view()[3][0], 17);
        assert_eq!(dst.as_view()[3][3], 42);
    }
}

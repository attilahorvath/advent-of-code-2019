use std::fmt;

#[derive(Copy, Clone, PartialEq)]
enum Pixel {
    Black = 0,
    White = 1,
    Transparent = 2,
    Invalid = 9,
}

impl From<char> for Pixel {
    fn from(c: char) -> Pixel {
        match c {
            x if x.to_digit(10) == Some(Pixel::Black as u32) => Pixel::Black,
            x if x.to_digit(10) == Some(Pixel::White as u32) => Pixel::White,
            x if x.to_digit(10) == Some(Pixel::Transparent as u32) => Pixel::Transparent,
            _ => Pixel::Invalid,
        }
    }
}

impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if *self == Pixel::White { '#' } else { ' ' })
    }
}

struct Layer {
    width: usize,
    pixels: Vec<Pixel>,
}

impl Layer {
    fn parse(data: &str, width: usize) -> Self {
        let pixels = data.chars().map(|c| c.into()).collect();

        Self { width, pixels }
    }

    fn count(&self, pixel: Pixel) -> usize {
        self.pixels.iter().filter(|&&p| p == pixel).count()
    }

    fn checksum(&self) -> usize {
        self.count(Pixel::White) * self.count(Pixel::Transparent)
    }

    fn pixel_at(&self, x: usize, y: usize) -> Pixel {
        self.pixels[y * self.width + x]
    }
}

pub struct Image {
    width: usize,
    height: usize,
    layers: Vec<Layer>,
}

impl Image {
    pub fn parse(data: &str, width: usize, height: usize) -> Self {
        let pixels = width * height;
        let depth = data.len() / pixels;

        let layers = (0..depth)
            .map(|l| Layer::parse(&data[l * pixels..(l + 1) * pixels], width))
            .collect();

        Self {
            width,
            height,
            layers,
        }
    }

    pub fn checksum(&self) -> usize {
        self.layers
            .iter()
            .min_by_key(|l| l.count(Pixel::Black))
            .unwrap()
            .checksum()
    }

    fn pixel_at(&self, x: usize, y: usize) -> Pixel {
        self.layers
            .iter()
            .map(|l| l.pixel_at(x, y))
            .find(|&p| p != Pixel::Transparent)
            .unwrap_or(Pixel::Transparent)
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.pixel_at(x, y))?;
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum() {
        let image = Image::parse("123456789012", 3, 2);

        assert_eq!(1, image.checksum());
    }

    #[test]
    fn draw() {
        let image = Image::parse(&"0222112222120000", 2, 2);

        assert_eq!(" #\n# \n", format!("{}", image));
    }
}

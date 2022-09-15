use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;
use crate::stl::face::Face;
use crate::stl::model::Model;
use crate::stl::point_3d::Point3D;
use crate::stl::volume::Volume;


// Writing to a file
impl Model {
    fn write_ascii<T: Write>(&self, writer: &mut T, resolution: f64) -> std::io::Result<()> {
        writeln!(writer, "solid plate")?;

        for volume in &self.volumes {
            for face in &volume.faces {
                let normal = face.get_normal();
                writeln!(writer, "  facet normal {}", normal.format_ascii_point())?;
                writeln!(writer, "    outer loop")?;

                for point in &face.v {
                    writeln!(writer, "       vertex {}", point.format_vertex(resolution))?
                }

                writeln!(writer, "    endloop")?;
                writeln!(writer, "  endfacet")?;
            }
        }

        writeln!(writer, "endsolid plate")
    }

    pub fn save_to_file_ascii<P: AsRef<Path>>(&self, filename: P, resolution: f64) -> std::io::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);
        self.write_ascii(&mut writer, resolution)?;
        writer.flush()
    }

    fn write_binary<T: Write>(&self, writer: &mut T, resolution: f64) -> std::io::Result<()> {
        let fake_stl_header: Vec<u8> = vec![0; 80];
        writer.write_all(&fake_stl_header)?;

        let face_count = (&self.volumes)
            .iter()
            .map(|vol| (&vol.faces).len())
            .sum::<usize>() as u32;

        writer.write_u32::<LittleEndian>(face_count)?;

        let flag: [u8; 2] = [0, 0];
        for volume in &self.volumes {
            for face in &volume.faces {
                let normal = face.get_normal();
                let points = [&normal, &face.v[0], &face.v[1], &face.v[2]];

                for point in points {
                    let x = (point.x/resolution) as f32;
                    let y = (point.y/resolution) as f32;
                    let z = (point.z/resolution) as f32;

                    writer.write_f32::<LittleEndian>(x)?;
                    writer.write_f32::<LittleEndian>(y)?;
                    writer.write_f32::<LittleEndian>(z)?;
                }

                writer.write_all(&flag)?;
            }
        }

        Ok(())
    }

    pub fn save_to_file_binary<P: AsRef<Path>>(&self, filename: P, resolution: f64) -> std::io::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);
        self.write_binary(&mut writer, resolution)?;
        writer.flush()
    }
}

// Reading from a file binary
impl Model {
    fn read_point_3d_binary<T: Read>(reader: &mut T, resolution: f64) -> std::io::Result<Point3D> {
        let x = resolution * reader.read_f32::<LittleEndian>()? as f64;
        let y = resolution * reader.read_f32::<LittleEndian>()? as f64;
        let z = resolution * reader.read_f32::<LittleEndian>()? as f64;
        Ok(Point3D {x, y, z})
    }

    fn load_stl_binary<T: Read>(reader: &mut T, resolution: f64) -> std::io::Result<Self> {
        let mut model = Model::new();
        let mut volume = Volume::new();

        let mut dummy_stl_header_buffer: [u8; 80] = [0; 80];
        reader.read_exact(&mut dummy_stl_header_buffer)?;

        let face_count = reader.read_u32::<LittleEndian>()?;

        let mut flags: [u8; 2] = [0, 0];

        for _ in 0..face_count {
            // Discard normal
            let _normal = Model::read_point_3d_binary(reader, resolution)?;

            let v0 = Model::read_point_3d_binary(reader, resolution)?;
            let v1 = Model::read_point_3d_binary(reader, resolution)?;
            let v2 = Model::read_point_3d_binary(reader, resolution)?;

            // Discard flag
            reader.read_exact(&mut flags)?;

            volume.add_face(Face::new(v0, v1, v2));
        }

        model.volumes.push(volume);
        Ok(model)
    }

    pub fn load_stl_file_binary<P: AsRef<Path>>(filename: P, resolution: f64) -> std::io::Result<Self> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        Model::load_stl_binary(&mut reader, resolution)
    }
}


impl Model {
    pub fn load_stl_file_ascii<P: AsRef<Path>>(filename: P, resolution: f64) -> std::io::Result<Self> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        Model::load_stl_ascii(&mut reader, resolution)
    }

    fn parse_ascii_vertex(line: &str, resolution: f64) -> Option<Point3D> {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 4 {
            return None
        }

        let x = resolution * parts[1].parse::<f64>().ok()?;
        let y = resolution * parts[2].parse::<f64>().ok()?;
        let z = resolution * parts[3].parse::<f64>().ok()?;

        Some(Point3D::new(x, y, z))
    }

    fn load_stl_ascii<T: BufRead>(reader: &mut T, resolution: f64) -> std::io::Result<Self> {
        let mut volume = Volume::new();

        let lines = reader
            .lines()
            .map(|line_result| line_result.unwrap())
            .filter_map(|line| {
                let trimmed_line = line.trim();
                if !trimmed_line.starts_with("vertex") {
                    return None
                }
                Some(Model::parse_ascii_vertex(trimmed_line, resolution))
            })
            .chunks(3);

        for chunk in &lines {
            // If we failed to parse a point, fail here
            let k: Vec<_> = chunk
                .map(|x| x.unwrap())
                .collect();

            if k.len() != 3 {
                todo!()
            }

            volume.add_face(Face::new(k[0], k[1], k[2]))
        }

        let mut model = Model::new();
        model.volumes.push(volume);
        Ok(model)
    }

}

impl Model {
    fn load_stl_file<P: AsRef<Path>>(filename: P, resolution: f64) -> std::io::Result<Self> {
        let mut f = File::open(filename.as_ref())?;

        let buf_len = 2048;
        let mut buf = Vec::with_capacity(buf_len);
        let n = f.read(&mut buf)?;
        let bytes = &buf.as_slice()[0..n];

        drop(f);

        let prefix = "solid".as_bytes();



        if n >= 5 {
            let x = &bytes[0..5];
            if x == prefix {
                return Model::load_stl_file_binary(filename, resolution)
            }
        }

        let printable_count = bytes
            .iter()
            .filter(|x| **x < 127)
            .count();

        if (printable_count as f64)/(n as f64) < 0.95
        {Model::load_stl_file_binary(filename, resolution)}
        else {Model::load_stl_file_ascii(filename, resolution)  }
    }
}
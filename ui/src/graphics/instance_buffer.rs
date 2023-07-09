use bytemuck::NoUninit;
use std::ops::Range;
use wgpu::util::DeviceExt;

pub struct InstanceBuffer<T: NoUninit> {
    instances: Vec<(usize, T)>,
    buffer: wgpu::Buffer,
    /// Updated index range
    updated: Option<Range<usize>>,
    /// Size of a one instance
    instance_size: usize,
    invalidate: bool,
    id_counter: usize
}

impl<T: NoUninit> InstanceBuffer<T> {
    pub fn new(device: &wgpu::Device, instances: Vec<T>) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        InstanceBuffer {
            instances: instances.iter().enumerate().map(|(i, ins)| (i, *ins)).collect::<Vec<_>>(),
            buffer,
            updated: None,
            instance_size: std::mem::size_of::<T>(),
            invalidate: false,
            id_counter: instances.len(),
        }
    }

    pub fn replace(&mut self, id: usize, instance: T) {
        let idx = self.instances.iter().position(|i| i.0 == id).unwrap();
        self.instances[idx].1 = instance;
        if self.invalidate {
            return;
        }
        if let Some(mut updated) = self.updated.take() {
            if !updated.contains(&idx) {
                if updated.start > idx {
                    updated.start = idx;
                }
                if updated.end <= idx {
                    updated.end = idx + 1;
                }
            }
            self.updated.insert(updated);
        } else {
            self.updated = Some(idx..(idx + 1));
        }
    }

    pub fn remove(&mut self, id: usize) {
        let idx = self.instances.iter().position(|i| i.0 == id).unwrap();
        self.instances.remove(idx);

        if self.invalidate {
            return;
        }

        if let Some(mut updated) = self.updated.take() {
            if !updated.contains(&idx) {
                if updated.start > idx {
                    updated.start = idx;
                }
                updated.end = self.instances.len();
            }
            self.updated.insert(updated);
        } else {
            self.updated = Some(idx..self.instances.len());
        }
    }

    pub fn add(&mut self, instance: T) -> usize {
        self.id_counter += 1;
        let next_id = self.id_counter;
        self.instances.push((next_id,instance));
        self.updated = None;
        self.invalidate = true;
        next_id
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn as_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub fn reset(&mut self) {
        self.instances = vec![];
        self.id_counter = 0;
        self.updated = None;
        self.invalidate = true;
    }

    pub fn get_ids(&self) -> Vec<usize> {
        self.instances.iter().map(|i|i.0).collect::<Vec<_>>()
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.invalidate {
            self.buffer.destroy();
            let instances = self.instances.iter().map(|i|i.1).collect::<Vec<_>>();
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
            self.invalidate = false;
            self.updated = None;
        }

        if let Some(updated) = self.updated.take() {
            let offset = updated.start * self.instance_size;
            let slice = self.instances[updated].iter().map(|i|i.1).collect::<Vec<_>>();
            queue.write_buffer(&self.buffer, offset as wgpu::BufferAddress, bytemuck::cast_slice(&slice));
        }
    }
}

use std::collections::HashSet;
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Default, Clone, PartialEq)]
pub struct DirectedGraph<T> {
    nodes: Vec<T>,
    succ: Vec<HashSet<usize>>,
    pred: Vec<HashSet<usize>>,
}

impl<T> DirectedGraph<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            succ: Vec::new(),
            pred: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            succ: Vec::with_capacity(capacity),
            pred: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn add(&mut self, value: T) -> usize {
        self.nodes.push(value);
        self.succ.push(HashSet::new());
        self.pred.push(HashSet::new());

        self.nodes.len() - 1
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.nodes.into_iter()
    }

    pub fn edges(&self) -> impl Iterator<Item = (usize, impl Iterator<Item = usize> + '_)> + '_ {
        self.succ
            .iter()
            .enumerate()
            .map(|(from, to)| (from, to.iter().copied()))
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        if from >= self.nodes.len() {
            panic!("out of bounds: {}", from);
        }
        if to >= self.nodes.len() {
            panic!("out of bounds: {}", to);
        }

        self.succ[from].insert(to);
        self.pred[to].insert(from);
    }

    pub fn succ(&self, index: usize) -> impl Iterator<Item = &T> + '_ {
        let edges = &self.succ[index];
        edges.iter().map(move |index| &self.nodes[*index])
    }

    pub fn succ_indexes(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        self.succ[index].iter().copied()
    }

    pub fn pred(&self, index: usize) -> impl Iterator<Item = &T> + '_ {
        let edges = &self.pred[index];
        edges.iter().map(move |index| &self.nodes[*index])
    }

    pub fn pred_indexes(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        self.pred[index].iter().copied()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes.get_mut(index)
    }
}

impl<T> Index<usize> for DirectedGraph<T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        &self.nodes[i]
    }
}

impl<T> IndexMut<usize> for DirectedGraph<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.nodes[i]
    }
}

impl<T: fmt::Debug> fmt::Debug for DirectedGraph<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Node<'a, T> {
            value: &'a T,
            pred: Vec<usize>,
            succ: Vec<usize>,
        }

        let mut ds = f.debug_struct("DirectedGraph");

        for i in 0..self.len() {
            let node = Node {
                value: &self[i],
                succ: self.succ_indexes(i).collect(),
                pred: self.pred_indexes(i).collect(),
            };

            ds.field(&format!("{}", i), &node);
        }

        ds.finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut graph = DirectedGraph::new();
        graph.add(3);

        assert_eq!(graph.nodes[0], 3);
        assert_eq!(graph.succ.len(), 1);
        assert_eq!(graph.pred.len(), 1);
    }

    #[test]
    fn test_pred() {
        let mut graph = DirectedGraph::new();
        let a = graph.add(30);
        let b = graph.add(10);
        let c = graph.add(25);
        let d = graph.add(29);
        graph.add_edge(a, b);
        graph.add_edge(a, c);
        graph.add_edge(b, c);
        graph.add_edge(c, d);

        let pred: Vec<&i32> = graph.pred(c).collect();
        assert!(pred.contains(&&30));
        assert!(pred.contains(&&10));

        let indexes: Vec<usize> = graph.pred_indexes(c).collect();
        assert!(indexes.contains(&a));
        assert!(indexes.contains(&b));
    }

    #[test]
    fn test_succ() {
        let mut graph = DirectedGraph::new();
        let a = graph.add(30);
        let b = graph.add(10);
        let c = graph.add(25);
        let d = graph.add(29);
        graph.add_edge(a, b);
        graph.add_edge(a, c);
        graph.add_edge(b, c);
        graph.add_edge(c, d);

        let succ: Vec<&i32> = graph.succ(a).collect();
        assert!(succ.contains(&&10));
        assert!(succ.contains(&&25));

        let indexes: Vec<usize> = graph.succ_indexes(a).collect();
        assert!(indexes.contains(&b));
        assert!(indexes.contains(&c));
    }
}

use std::collections::HashSet;
use speedy2d::dimen::Vec2;
use crate::edge::Edge;
use crate::polygon::is_point_in_polygon;
use crate::vec::cross_product;

#[derive(Copy, Clone)]
struct IntersectionInfo<'a> {
    id: usize,
    this_edge: &'a Edge,
    other_edge: &'a Edge,
    point: Vec2,
}

struct IntersectionEdgeIdPair {
    sub_id: usize,
    clip_id: usize
}

struct ClippingData<'a> {
    intersect_list: Vec<IntersectionEdgeIdPair>,
    intersect_s: Vec<Vec<IntersectionInfo<'a>>>,
    intersect_c: Vec<Vec<IntersectionInfo<'a>>>,
    subject_polygon: Vec<Edge>,
    clipping_polygon: Vec<Edge>,
    visited: HashSet<usize>,
}

fn search_edge (d: &mut ClippingData, walk_subject_edge: bool, edge_index: usize, intersect_id: usize) -> Vec<Edge> {
    if d.visited.contains(&intersect_id) {
        return vec![]
    } else {
        d.visited.insert(intersect_id);
    }
    return if walk_subject_edge {
        let cur_inter_pos = d.intersect_s[edge_index].iter().position(|x| x.id == intersect_id).unwrap();
        let cur_inter = d.intersect_s[edge_index][cur_inter_pos];
        if let Some(out_inter) = d.intersect_s[edge_index].get(cur_inter_pos + 1) {
            // case 1. out vertex also on this edge
            let cur_edge = Edge { from: cur_inter.point, to: out_inter.point };
            let rest = search_edge(d, false, d.intersect_list[out_inter.id].clip_id, out_inter.id);
            vec![cur_edge].into_iter().chain(rest.into_iter()).collect()
        } else {
            // case 2. out vertex is on the next edges
            let mut cur_edges = vec![Edge { from: cur_inter.point, to: d.subject_polygon[edge_index].to }];
            let mut rest = vec![];
            let mut expected_start_pos = d.subject_polygon[edge_index].to;
            for i in ((edge_index + 1)..(d.subject_polygon.len())).chain(0..edge_index) {
                // not following current edge
                if (d.subject_polygon[i].from - expected_start_pos).magnitude_squared() > f32::EPSILON {
                    continue
                }
                if d.intersect_s[i].is_empty() {
                    cur_edges.push(d.subject_polygon[i]);
                    expected_start_pos = d.subject_polygon[i].to;
                } else {
                    let it = d.intersect_s[i].first().unwrap();
                    cur_edges.push(Edge { from: d.subject_polygon[i].from, to: it.point });
                    rest = search_edge(d, false, d.intersect_list[it.id].clip_id, it.id);
                    break
                }
            }
            cur_edges.into_iter().chain(rest.into_iter()).collect()
        }
    } else {
        let cur_inter_pos = d.intersect_c[edge_index].iter().position(|x| x.id == intersect_id).unwrap();
        let cur_inter = d.intersect_c[edge_index][cur_inter_pos];
        if let Some(in_inter) = d.intersect_c[edge_index].get(cur_inter_pos + 1) {
            // case 1. in vertex also on this edge
            let cur_edge = Edge { from: cur_inter.point, to: in_inter.point };
            let rest = search_edge(d, true, d.intersect_list[in_inter.id].sub_id, in_inter.id);
            vec![cur_edge].into_iter().chain(rest.into_iter()).collect()
        } else {
            // case 2. in vertex is on the next edges
            let mut cur_edges = vec![Edge { from: cur_inter.point, to: d.clipping_polygon[edge_index].to }];
            let mut rest = vec![];
            let mut expected_start_pos = d.clipping_polygon[edge_index].to;
            for i in ((edge_index + 1)..(d.clipping_polygon.len())).chain(0..edge_index) {
                // not following current edge
                if (d.clipping_polygon[i].from - expected_start_pos).magnitude_squared() > f32::EPSILON {
                    continue
                }
                if d.intersect_c[i].is_empty() {
                    cur_edges.push(d.clipping_polygon[i]);
                    expected_start_pos = d.clipping_polygon[i].to;
                } else {
                    let it = d.intersect_c[i].first().unwrap();
                    cur_edges.push(Edge { from: d.clipping_polygon[i].from, to: it.point });
                    rest = search_edge(d, true, d.intersect_list[it.id].sub_id, it.id);
                    break
                }
            }
            cur_edges.into_iter().chain(rest.into_iter()).collect()
        }
    }
}

pub fn clip_polygon (subject_polygon: &[Edge], clipping_polygon: &[Edge]) -> Vec<Edge> {
    let mut d = ClippingData {
        intersect_list: vec![],
        intersect_s: vec![vec![]; subject_polygon.len()],
        intersect_c: vec![vec![]; clipping_polygon.len()],
        subject_polygon: subject_polygon.to_vec(),
        clipping_polygon: clipping_polygon.to_vec(),
        visited: HashSet::new(),
    };

    for (i, e_sub) in subject_polygon.iter().enumerate() {
        for (j, e_clip) in clipping_polygon.iter().enumerate() {
            if let Some(intersection) = e_sub.intersect_with(e_clip) {
                d.intersect_s[i].push(IntersectionInfo {
                    id: d.intersect_list.len(),
                    this_edge: e_sub,
                    other_edge: e_clip,
                    point: intersection,
                });
                d.intersect_c[j].push(IntersectionInfo {
                    id: d.intersect_list.len(),
                    this_edge: e_clip,
                    other_edge: e_sub,
                    point: intersection,
                });
                d.intersect_list.push(IntersectionEdgeIdPair { sub_id: i, clip_id: j });
            }
        }
    }

    if d.intersect_list.is_empty() {
        // one polygon completely lies in another
        // take a vertex from main polygon. see if it lies in the clipping polygon
        // yes -> return main polygon; no -> return clipping polygon
        let all_visible = is_point_in_polygon(d.subject_polygon.first().unwrap().from, &d.clipping_polygon);
        if all_visible { d.subject_polygon } else { d.clipping_polygon }
    } else {
        let sort_func = |a: &IntersectionInfo, b: &IntersectionInfo|
            (a.point - a.this_edge.from).magnitude_squared()
                .total_cmp(&(b.point - b.this_edge.from).magnitude_squared());
        d.intersect_s.iter_mut().for_each(|x| x.sort_by(sort_func));
        d.intersect_c.iter_mut().for_each(|x| x.sort_by(sort_func));

        let mut res = vec![];
        let intersect_s_clone = d.intersect_s.clone();
        for (i, it_list) in intersect_s_clone.iter().enumerate() {
            for it in it_list.iter() {
                if !d.visited.contains(&it.id) {
                    let is_in_edge = cross_product(&it.this_edge.get_vector(), &it.other_edge.get_vector()) > 0.0;
                    if is_in_edge {
                        res.extend( search_edge(&mut d, true, i, it.id))
                    } else {
                        let j = d.intersect_list[it.id].clip_id;
                        res.extend(search_edge(&mut d, false, j, it.id));
                    }
                }
            }
        }
        res
    }
}
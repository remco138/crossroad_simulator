use itertools::Itertools;
use permutohedron::Heap;

// Copied values
//
fn combine<'a, 'b, T>(
		xs: &'a Vec<&'b T>,
		ys: &'a Vec<&'b T>) -> Vec<Vec<&'b T>> {

	xs.iter()
	  .cartesian_product(ys.iter())
      .map(|(a,b)| vec![*a, *b]).collect()
}

fn add<'a, 'b, T>(
		mut to: Vec<Vec<&'a T>>,
		to_add: &'b Vec<&'a T>) -> Vec<Vec<&'a T>> {

	to.drain(..)
	  .cartesian_product(to_add.iter())
	  .map(|(mut a, b)| { a.push(*b); a }).collect()
}

pub fn combine_list<'a, T>(mut a: Vec<Vec<&'a T>>) -> Vec<Vec<&'a T>> {

	match a.len() {
		0 => return vec![],
		1 => return a,
		_ => {},
	}

	let first = a.pop().unwrap();
	let secon = a.pop().unwrap();
	let start = combine(&first, &secon);

	a.iter().fold(start, |acc,b| add(acc, b))
}

pub fn all_possibilities<'a, T>(mut all: Vec<Vec<&'a T>>) -> Vec<Vec<&'a T>> {
	 Heap::new(&mut all).flat_map(|i| combine_list(i)).collect()
}

// References
//
fn combine_v<T: Clone>(xs: &Vec<T>, ys: &Vec<T>) -> Vec<Vec<T>> {
	xs.iter()
	  .cartesian_product(ys.iter())
      .map(|(a,b)| vec![a.clone(), b.clone()]).collect()
}

fn add_v<T: Clone>(mut to: Vec<Vec<T>>, to_add: &Vec<T>) -> Vec<Vec<T>> {
	to.drain(..)
	  .cartesian_product(to_add.iter())
	  .map(|(mut a, b)| { a.push(b.clone()); a }).collect()
}

pub fn combine_list_v<T: Clone>(mut a: Vec<Vec<T>>) -> Vec<Vec<T>> {

	match a.len() {
		0 => return vec![],
		1 => return a,
		_ => {},
	}


	let first = a.pop().unwrap();
	let secon = a.pop().unwrap();
	let start = combine_v(&first, &secon);

	a.iter().fold(start, |acc,b| add_v(acc, b))
}

pub fn all_possibilities_v<T: Clone>(mut all: Vec<Vec<T>>) -> Vec<Vec<T>> {
	 Heap::new(&mut all).flat_map(|i| combine_list_v(i)).collect()
}

fn main() {
    let mut data = vec![1, 2, 3];
    
    if data.len() > 0 {
        println!("Data has {} elements", data.len());
        
        for (index, value) in data.iter().enumerate() {
            println!("Element at index {}: {}", index, value);
        }
    }
    
    let matrix = [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ];
    
    let result = calculate_sum(&matrix);
    println!("Sum: {}", result);
}

fn calculate_sum(matrix: &[[i32; 3]; 3]) -> i32 {
    let mut sum = 0;
    for row in matrix {
        for &value in row {
            sum += value;
        }
    }
    sum
}
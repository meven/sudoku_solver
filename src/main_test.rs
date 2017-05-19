
#[cfg(test)]
mod tests {

    #[test]
    fn test_get_head_of_block() {
        assert!(get_head_of_block(1) == 0);
        assert!(get_head_of_block(11) == 0);
        assert!(get_head_of_block(70) == 60);
        assert!(get_head_of_block(78) == 60);
        assert!(get_head_of_block(54) == 54);
        assert!(get_head_of_block(56) == 54);
        assert!(get_head_of_block(49) == 30);
        assert!(get_head_of_block(47) == 27);
        assert!(get_head_of_block(28) == 27);
        assert!(get_head_of_block(35) == 33);
        assert!(get_head_of_block(26) == 6);
        assert!(get_head_of_block(32) == 30);
        assert!(get_head_of_block(17) == 6);
    }


    #[test]
    fn test_get_possible_values() {
        let grid_string = r#"
            1 _ _   _ 3 _   _ _ 9
            _ _ _   _ 2 _   _ _ _
            3 _ 7   _ _ _   _ 5 _

            _ _ _   _ _ _   _ _ _
            _ _ 4   _ _ _   _ _ _
            7 _ _   _ _ _   _ _ _

            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _
            _ 8 _   _ _ _   _ _ _"#;

        let grid: Grid = parse_grid(grid_string);

        println!("{:?}", get_possible_values(grid, 1));
        assert_eq!(vec![2, 4, 5, 6], get_possible_values(grid, 1));

        // /
        // let mut grid: Grid = [None; 81];
        // for i in 0..81 {
        // grid[i] = Some(i);
        // }
        // print_grid(grid);
        // assert!(false);
        //
    }
}

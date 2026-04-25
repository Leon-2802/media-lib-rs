#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub test: i32
}

impl Test {
    pub fn new(test: i32) -> Self {
        Self { test }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentTest {
    pub my_test: Test
}

impl ParentTest {
    pub fn new(my_test: Test) -> Self {
        Self { my_test }
    }
}
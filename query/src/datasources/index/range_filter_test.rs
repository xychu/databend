// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::collections::HashMap;

use common_datavalues::prelude::*;
use common_exception::Result;
use common_planners::*;

use crate::datasources::index::range_filter::build_verifiable_expr;
use crate::datasources::index::range_filter::StatColumns;
use crate::datasources::index::RangeFilter;
use crate::datasources::table::fuse::util::BlockStats;
use crate::datasources::table::fuse::ColStats;

#[test]
fn test_range_filter() -> Result<()> {
    let schema = DataSchemaRefExt::create(vec![
        DataField::new("a", DataType::Int64, false),
        DataField::new("b", DataType::Int32, false),
    ]);

    let mut stats: BlockStats = HashMap::new();
    stats.insert(0u32, ColStats {
        min: DataValue::Int32(Some(1)),
        max: DataValue::Int32(Some(20)),
        null_count: 1,
    });
    stats.insert(1u32, ColStats {
        min: DataValue::Int32(Some(3)),
        max: DataValue::Int32(Some(10)),
        null_count: 0,
    });

    #[allow(dead_code)]
    struct Test {
        name: &'static str,
        expr: Expression,
        expect: bool,
    }

    let tests: Vec<Test> = vec![
        Test {
            name: "a < 1 and b > 3",
            expr: col("a").lt(lit(1)).and(col("b").gt(lit(3))),
            expect: false,
        },
        Test {
            name: "1 > -a or 3 >= b",
            expr: lit(1).gt(neg(col("a"))).or(lit(3).gt_eq(col("b"))),
            expect: true,
        },
        Test {
            name: "a = 1 and b != 3",
            expr: col("a").eq(lit(1)).and(col("b").not_eq(lit(3))),
            expect: true,
        },
        Test {
            name: "a is null",
            expr: Expression::create_scalar_function("isNull", vec![col("a")]),
            expect: true,
        },
        Test {
            name: "a is not null",
            expr: Expression::create_scalar_function("isNotNull", vec![col("a")]),
            expect: true,
        },
        Test {
            name: "null",
            expr: Expression::create_literal(DataValue::Null),
            expect: false,
        },
        Test {
            name: "b >= 0 and c like '%sys%'",
            expr: col("b")
                .gt_eq(lit(0))
                .and(Expression::create_binary_expression("like", vec![
                    col("c"),
                    lit("%sys%".as_bytes()),
                ])),
            expect: true,
        },
    ];

    for test in tests {
        let prune = RangeFilter::try_create(&test.expr, schema.clone())?;
        let actual = prune.eval(&stats)?;
        assert_eq!(test.expect, actual, "{:#?}", test.name);
    }

    Ok(())
}

#[test]
fn test_build_verifiable_function() -> Result<()> {
    let schema = DataSchemaRefExt::create(vec![
        DataField::new("a", DataType::Int64, false),
        DataField::new("b", DataType::Int32, false),
        DataField::new("c", DataType::String, false),
    ]);

    #[allow(dead_code)]
    struct Test {
        name: &'static str,
        expr: Expression,
        expect: &'static str,
    }

    let tests: Vec<Test> = vec![
        Test {
            name: "a < 1 and b > 3",
            expr: col("a").lt(lit(1)).and(col("b").gt(lit(3))),
            expect: "((min_a < 1) and (max_b > 3))",
        },
        Test {
            name: "1 > -a or 3 >= b",
            expr: lit(1).gt(neg(col("a"))).or(lit(3).gt_eq(col("b"))),
            expect: "(((- max_a) < 1) or (min_b <= 3))",
        },
        Test {
            name: "a = 1 and b != 3",
            expr: col("a").eq(lit(1)).and(col("b").not_eq(lit(3))),
            expect: "(((min_a <= 1) and (max_a >= 1)) and ((min_b != 3) or (max_b != 3)))",
        },
        Test {
            name: "a is null",
            expr: Expression::create_scalar_function("isNull", vec![col("a")]),
            expect: "(nulls_a > 0)",
        },
        Test {
            name: "a is not null",
            expr: Expression::create_scalar_function("isNotNull", vec![col("a")]),
            expect: "isNotNull(min_a)",
        },
        Test {
            name: "b >= 0 and c like '%sys%'",
            expr: col("b")
                .gt_eq(lit(0))
                .and(Expression::create_binary_expression("like", vec![
                    col("c"),
                    lit("%sys%".as_bytes()),
                ])),
            expect: "((max_b >= 0) and true)",
        },
    ];

    for test in tests {
        let mut stat_columns: StatColumns = Vec::new();
        let res = build_verifiable_expr(&test.expr, schema.clone(), &mut stat_columns);
        let actual = format!("{:?}", res);
        assert_eq!(test.expect, actual, "{:#?}", test.name);
    }

    Ok(())
}
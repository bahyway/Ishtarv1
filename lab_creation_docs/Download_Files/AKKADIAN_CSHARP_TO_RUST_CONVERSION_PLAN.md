# Akkadian DSL v3.4 - C# to Rust Conversion Plan
## Converting Existing Working Implementation to Rust

---

## 🎯 **SITUATION:**

You have:
- ✅ **Akkadian v3.4** (working in C#)
- ✅ **Akkadian v3.3** (working in C#)
- ✅ **Akkadian v3.2** (working in C#)
- ✅ **Complete parser, compiler, and runtime**
- ✅ **ColorID integration**
- ✅ **Fuzzy logic engine**
- ✅ **25+ language constructs**

You need:
- 🎯 Port everything to Rust for BDBWay v1.0
- 🎯 Integrate with pgrx (PostgreSQL extension)
- 🎯 Maintain all C# features
- 🎯 Add performance improvements

---

## 📅 **REVISED TIMELINE (Much Faster!):**

**Original Plan:** 6 weeks (building from scratch)
**New Plan:** 3-4 weeks (porting existing code)

### **Week 1: Parser Port (C# → Rust)**
- Day 1-2: Port lexer/tokenizer
- Day 3-4: Port AST definitions
- Day 5: Port parser logic

### **Week 2: Compiler Port**
- Day 6-8: Port SQL compiler
- Day 9-10: Port validation & optimization

### **Week 3: Fuzzy Logic & ColorID**
- Day 11-13: Port fuzzy logic engine
- Day 14-15: Port ColorID integration

### **Week 4: Testing & Integration**
- Day 16-18: Port test suite
- Day 19-20: BDBWay integration

---

## 🔄 **C# TO RUST MAPPING:**

### **1. Type System Mapping**

| C# Type | Rust Equivalent | Notes |
|---------|-----------------|-------|
| `class AkkadianParser` | `struct AkkadianParser` | Structs in Rust |
| `interface INode` | `trait Node` | Traits = interfaces |
| `enum NodeType` | `enum NodeType` | Same concept |
| `string` | `String` or `&str` | Owned vs borrowed |
| `List<T>` | `Vec<T>` | Dynamic arrays |
| `Dictionary<K,V>` | `HashMap<K,V>` | Hash maps |
| `decimal` | `f64` | Floating point |
| `byte` | `u8` | Unsigned 8-bit |
| `Guid` | `Uuid` | Use uuid crate |

### **2. Language Feature Mapping**

| C# Feature | Rust Equivalent | Example |
|------------|-----------------|---------|
| **LINQ** | **Iterator chains** | `.Where()` → `.filter()` |
| **Properties** | **Methods** | `get Name()` → `name()` |
| **Nullable<T>** | **Option<T>** | `int?` → `Option<i32>` |
| **Try-Catch** | **Result<T,E>** | `try/catch` → `match result` |
| **async/await** | **async/await** | Same syntax! |
| **Generics** | **Generics** | Similar syntax |

### **3. Parser Implementation Mapping**

**C# (Your Existing Code):**
```csharp
public class AkkadianParser {
    private readonly Tokenizer _tokenizer;
    private Token _currentToken;
    
    public ASTNode Parse(string source) {
        _tokenizer = new Tokenizer(source);
        _currentToken = _tokenizer.NextToken();
        return ParseStatement();
    }
    
    private ASTNode ParseStatement() {
        switch (_currentToken.Type) {
            case TokenType.SEEK:
                return ParseSeekQuery();
            case TokenType.FIND:
                return ParseFindQuery();
            default:
                throw new ParseException($"Unexpected token: {_currentToken}");
        }
    }
}
```

**Rust (Converted):**
```rust
pub struct AkkadianParser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
}

impl<'a> AkkadianParser<'a> {
    pub fn parse(source: &'a str) -> Result<ASTNode, ParseError> {
        let mut tokenizer = Tokenizer::new(source);
        let current_token = tokenizer.next_token()?;
        
        Self { tokenizer, current_token }.parse_statement()
    }
    
    fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        match self.current_token.token_type {
            TokenType::Seek => self.parse_seek_query(),
            TokenType::Find => self.parse_find_query(),
            _ => Err(ParseError::UnexpectedToken(self.current_token.clone())),
        }
    }
}
```

---

## 📋 **STEP-BY-STEP CONVERSION PROCESS:**

### **Phase 1: Week 1 - Lexer & Parser (Days 1-5)**

#### **Day 1: Setup & Token Types**

**C# Location:** Look for your `Token.cs` or `TokenType.cs`

**Rust Implementation:**
```rust
// akkadian-dsl/src/tokens.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Seek,
    Find,
    Traverse,
    Aggregate,
    Filter,
    Transform,
    
    // ColorID specific
    ColorIdGenerator,
    QualityScore,
    NodeClass,
    
    // Operators
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    FuzzyMatch,
    
    // Literals
    Identifier(String),
    Number(f64),
    String(String),
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}
```

**TODO for Day 1:**
- [ ] Copy your C# TokenType enum
- [ ] Convert each token type to Rust enum
- [ ] Add position tracking (line, column)

---

#### **Day 2: Lexer/Tokenizer**

**C# Pattern (Your Code):**
```csharp
public class Tokenizer {
    private string _source;
    private int _position;
    
    public Token NextToken() {
        SkipWhitespace();
        
        if (IsAtEnd()) return new Token(TokenType.EOF);
        
        char c = Advance();
        
        if (IsAlpha(c)) return Identifier();
        if (IsDigit(c)) return Number();
        if (c == '"') return String();
        
        // Operators...
        return new Token(TokenType.Unknown);
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/lexer.rs

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();
        
        let ch = match self.peek() {
            Some(c) => *c,
            None => return Ok(Token::eof()),
        };
        
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            '0'..='9' => self.number(),
            '"' => self.string(),
            '(' => self.single_char_token(TokenType::LeftParen),
            ')' => self.single_char_token(TokenType::RightParen),
            // ... more operators
            _ => Err(LexError::UnexpectedChar(ch)),
        }
    }
    
    fn identifier(&mut self) -> Result<Token, LexError> {
        let start = self.current_pos;
        
        while let Some(&ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start..self.current_pos];
        let token_type = self.keyword_or_identifier(text);
        
        Ok(Token {
            token_type,
            lexeme: text.to_string(),
            line: self.line,
            column: self.column,
        })
    }
    
    fn keyword_or_identifier(&self, text: &str) -> TokenType {
        match text {
            "SEEK" => TokenType::Seek,
            "FIND" => TokenType::Find,
            "COLORID_GENERATOR" => TokenType::ColorIdGenerator,
            // ... more keywords
            _ => TokenType::Identifier(text.to_string()),
        }
    }
}
```

**TODO for Day 2:**
- [ ] Port your C# Tokenizer to Rust Lexer
- [ ] Add all keyword mappings
- [ ] Handle whitespace, comments
- [ ] Add error handling

---

#### **Day 3-4: AST Definitions**

**C# Pattern:**
```csharp
public abstract class ASTNode {
    public abstract void Accept(IVisitor visitor);
}

public class SeekQuery : ASTNode {
    public List<Condition> Conditions { get; set; }
    public int? Limit { get; set; }
    
    public override void Accept(IVisitor visitor) {
        visitor.Visit(this);
    }
}

public class Condition {
    public string Field { get; set; }
    public Operator Op { get; set; }
    public object Value { get; set; }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/ast.rs

#[derive(Debug, Clone)]
pub enum ASTNode {
    SeekQuery(SeekQuery),
    FindQuery(FindQuery),
    TraverseQuery(TraverseQuery),
    AggregateQuery(AggregateQuery),
    ColorIdGenerator(ColorIdGenerator),
}

#[derive(Debug, Clone)]
pub struct SeekQuery {
    pub conditions: Vec<Condition>,
    pub limit: Option<u32>,
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: Operator,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    FuzzyMatch { threshold: f64 },
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
}

// Visitor pattern in Rust
pub trait Visitor {
    fn visit_seek_query(&mut self, query: &SeekQuery);
    fn visit_find_query(&mut self, query: &FindQuery);
}
```

**TODO for Days 3-4:**
- [ ] Port all your C# AST node types
- [ ] Convert inheritance to enum-based AST
- [ ] Add all query types (SEEK, FIND, TRAVERSE, etc.)
- [ ] Port ColorID-specific nodes

---

#### **Day 5: Parser Logic**

**C# Pattern:**
```csharp
public class Parser {
    private Token[] _tokens;
    private int _current;
    
    public ASTNode Parse() {
        return ParseStatement();
    }
    
    private SeekQuery ParseSeekQuery() {
        Expect(TokenType.SEEK);
        var conditions = ParseConditions();
        var limit = ParseLimit();
        return new SeekQuery { Conditions = conditions, Limit = limit };
    }
    
    private List<Condition> ParseConditions() {
        var conditions = new List<Condition>();
        while (!IsAtEnd() && _tokens[_current].Type != TokenType.LIMIT) {
            conditions.Add(ParseCondition());
        }
        return conditions;
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/parser.rs

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_statement()
    }
    
    fn parse_seek_query(&mut self) -> Result<SeekQuery, ParseError> {
        self.expect(TokenType::Seek)?;
        let conditions = self.parse_conditions()?;
        let limit = self.parse_limit()?;
        
        Ok(SeekQuery {
            conditions,
            limit,
            order_by: None,
        })
    }
    
    fn parse_conditions(&mut self) -> Result<Vec<Condition>, ParseError> {
        let mut conditions = Vec::new();
        
        while !self.is_at_end() && !self.check(TokenType::Limit) {
            conditions.push(self.parse_condition()?);
        }
        
        Ok(conditions)
    }
    
    fn expect(&mut self, token_type: TokenType) -> Result<Token, ParseError> {
        if self.check(token_type.clone()) {
            Ok(self.advance())
        } else {
            Err(ParseError::ExpectedToken {
                expected: token_type,
                found: self.peek().clone(),
            })
        }
    }
}
```

---

### **Phase 2: Week 2 - Compiler (Days 6-10)**

#### **Day 6-8: SQL Compiler**

**C# Pattern:**
```csharp
public class SQLCompiler {
    public string CompileToSQL(ASTNode node) {
        return node switch {
            SeekQuery seek => CompileSeekQuery(seek),
            FindQuery find => CompileFindQuery(find),
            _ => throw new NotSupportedException()
        };
    }
    
    private string CompileSeekQuery(SeekQuery query) {
        var sql = new StringBuilder("SELECT * FROM nodes WHERE ");
        
        for (int i = 0; i < query.Conditions.Count; i++) {
            if (i > 0) sql.Append(" AND ");
            sql.Append(CompileCondition(query.Conditions[i]));
        }
        
        if (query.Limit.HasValue) {
            sql.Append($" LIMIT {query.Limit.Value}");
        }
        
        return sql.ToString();
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/compiler/sql.rs

pub struct SQLCompiler;

impl SQLCompiler {
    pub fn compile(node: &ASTNode) -> Result<String, CompileError> {
        match node {
            ASTNode::SeekQuery(seek) => Self::compile_seek_query(seek),
            ASTNode::FindQuery(find) => Self::compile_find_query(find),
            _ => Err(CompileError::UnsupportedNode),
        }
    }
    
    fn compile_seek_query(query: &SeekQuery) -> Result<String, CompileError> {
        let mut sql = String::from("SELECT * FROM nodes WHERE ");
        
        for (i, condition) in query.conditions.iter().enumerate() {
            if i > 0 {
                sql.push_str(" AND ");
            }
            sql.push_str(&Self::compile_condition(condition)?);
        }
        
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        Ok(sql)
    }
    
    fn compile_condition(condition: &Condition) -> Result<String, CompileError> {
        let field = &condition.field;
        let value = Self::compile_value(&condition.value)?;
        
        let sql = match &condition.operator {
            Operator::Equal => format!("{} = {}", field, value),
            Operator::GreaterThan => format!("{} > {}", field, value),
            Operator::FuzzyMatch { threshold } => {
                format!("fuzzy_match({}, {}) > {}", field, value, threshold)
            }
            // ... more operators
        };
        
        Ok(sql)
    }
}
```

---

#### **Day 9-10: ColorID Integration**

**C# Pattern:**
```csharp
public class ColorIDCompiler {
    public string CompileColorIDQuery(ColorIDGenerator node) {
        return $@"
            SELECT bdb_generate_colorid(
                uuid: {node.UUID},
                domain: {node.DomainID},
                quality: {node.QualityScore},
                temporal: {node.TemporalValue}
            )";
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/compiler/colorid.rs

pub struct ColorIDCompiler;

impl ColorIDCompiler {
    pub fn compile(node: &ColorIdGenerator) -> Result<String, CompileError> {
        Ok(format!(
            "SELECT bdb_generate_colorid(uuid := '{}', domain := {}, quality := {}, temporal := {})",
            node.uuid,
            node.domain_id,
            node.quality_score,
            node.temporal_value
        ))
    }
    
    pub fn compile_quality_filter(threshold: u8) -> String {
        format!("get_byte(id, 13) >= {}", threshold)
    }
    
    pub fn compile_gem_query() -> String {
        String::from("get_byte(id, 13) >= 200")
    }
}
```

---

### **Phase 3: Week 3 - Fuzzy Logic (Days 11-15)**

**C# Pattern:**
```csharp
public class FuzzyLogicEngine {
    public double EvaluateMembership(double value, FuzzySet set) {
        // Triangular membership function
        if (value <= set.Left) return 0.0;
        if (value >= set.Right) return 0.0;
        if (value == set.Center) return 1.0;
        
        if (value < set.Center) {
            return (value - set.Left) / (set.Center - set.Left);
        } else {
            return (set.Right - value) / (set.Right - set.Center);
        }
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/src/fuzzy/engine.rs

pub struct FuzzyEngine;

impl FuzzyEngine {
    pub fn evaluate_membership(value: f64, set: &FuzzySet) -> f64 {
        // Triangular membership function
        if value <= set.left || value >= set.right {
            return 0.0;
        }
        
        if value == set.center {
            return 1.0;
        }
        
        if value < set.center {
            (value - set.left) / (set.center - set.left)
        } else {
            (set.right - value) / (set.right - set.center)
        }
    }
    
    pub fn fuzzy_and(a: f64, b: f64) -> f64 {
        a.min(b)
    }
    
    pub fn fuzzy_or(a: f64, b: f64) -> f64 {
        a.max(b)
    }
}

#[derive(Debug, Clone)]
pub struct FuzzySet {
    pub left: f64,
    pub center: f64,
    pub right: f64,
}
```

---

### **Phase 4: Week 4 - Testing & Integration (Days 16-20)**

#### **Day 16-18: Port Test Suite**

**C# Pattern:**
```csharp
[TestClass]
public class AkkadianParserTests {
    [TestMethod]
    public void TestSeekQuery() {
        var source = "SEEK name = 'John' LIMIT 10";
        var parser = new AkkadianParser();
        var ast = parser.Parse(source);
        
        Assert.IsInstanceOfType(ast, typeof(SeekQuery));
        var seek = ast as SeekQuery;
        Assert.AreEqual(1, seek.Conditions.Count);
        Assert.AreEqual(10, seek.Limit);
    }
}
```

**Rust Conversion:**
```rust
// akkadian-dsl/tests/parser_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seek_query() {
        let source = "SEEK name = 'John' LIMIT 10";
        let parser = AkkadianParser::new();
        let ast = parser.parse(source).unwrap();
        
        match ast {
            ASTNode::SeekQuery(seek) => {
                assert_eq!(seek.conditions.len(), 1);
                assert_eq!(seek.limit, Some(10));
            }
            _ => panic!("Expected SeekQuery"),
        }
    }
    
    #[test]
    fn test_colorid_generator() {
        let source = "COLORID_GENERATOR uuid='...' quality=240";
        let ast = parse(source).unwrap();
        
        match ast {
            ASTNode::ColorIdGenerator(gen) => {
                assert_eq!(gen.quality_score, 240);
            }
            _ => panic!("Expected ColorIdGenerator"),
        }
    }
}
```

---

## 🎯 **PRACTICAL CONVERSION WORKFLOW:**

### **Step 1: Find Your C# Files**
```
Your C# Project Structure:
├── Akkadian.Parser/
│   ├── Token.cs
│   ├── Tokenizer.cs
│   ├── Parser.cs
│   └── ASTNodes.cs
├── Akkadian.Compiler/
│   ├── SQLCompiler.cs
│   ├── CSharpCompiler.cs
│   └── Optimizer.cs
├── Akkadian.FuzzyLogic/
│   └── FuzzyEngine.cs
└── Akkadian.Tests/
    └── ParserTests.cs
```

### **Step 2: Convert File by File**
```
Week 1 Schedule:
Monday:    Token.cs → tokens.rs
Tuesday:   Tokenizer.cs → lexer.rs
Wednesday: ASTNodes.cs → ast.rs
Thursday:  Parser.cs → parser.rs (Part 1)
Friday:    Parser.cs → parser.rs (Part 2)
```

### **Step 3: Test As You Go**
```bash
# After each file conversion:
cargo test -p akkadian-dsl

# Run specific test:
cargo test test_seek_query

# Check compilation:
cargo check -p akkadian-dsl
```

---

## 📚 **HELPFUL RESOURCES:**

### **C# to Rust Cheat Sheet:**
```
C# String Methods → Rust Equivalents:
string.Contains()    → str.contains()
string.StartsWith()  → str.starts_with()
string.Split()       → str.split()
string.Trim()        → str.trim()
string.ToLower()     → str.to_lowercase()
StringBuilder        → String with push_str()

C# Collections → Rust Equivalents:
List<T>             → Vec<T>
Dictionary<K,V>     → HashMap<K,V>
HashSet<T>          → HashSet<T>
Queue<T>            → VecDeque<T>

C# LINQ → Rust Iterators:
.Where(x => x > 5)  → .filter(|x| *x > 5)
.Select(x => x * 2) → .map(|x| x * 2)
.First()            → .next()
.Count()            → .count()
.Any()              → .any()
.All()              → .all()
```

---

## ✅ **ACTION PLAN:**

### **This Week (Week 1):**
```bash
# Day 1-2: Setup & Tokens
cd /workspace/bahyway-fourpillarsway/akkadian-dsl/src

# Find your C# Token.cs file
# Convert to tokens.rs
# Test: cargo test

# Day 3-4: Lexer
# Find your C# Tokenizer.cs
# Convert to lexer.rs
# Test: cargo test

# Day 5: AST
# Find your C# ASTNodes.cs
# Convert to ast.rs
# Test: cargo test
```

---

## 🎯 **EXPECTED OUTCOME:**

**After 4 Weeks:**
```
✅ Akkadian DSL v3.4 fully ported to Rust
✅ All features from C# version working
✅ Integrated with BDBWay PostgreSQL extension
✅ 25+ language constructs operational
✅ ColorID support complete
✅ Fuzzy logic engine working
✅ Test suite passing
✅ Ready for production use
```

---

## 🚀 **NEXT STEPS:**

1. **Share your C# source files** (or key snippets)
2. **I'll create exact Rust equivalents**
3. **Start with tokens.rs tomorrow**
4. **Port incrementally, test continuously**

**Ready to start the conversion?** 💪🦀

Let me know which C# file you want to convert first!

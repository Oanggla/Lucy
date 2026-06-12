# 習題 2 -- Lucy 程式語言與棧式虛擬機器實作 (Rust 版)

本專案使用 **Rust** 重新設計並實現了一個全新的程式語言：**Lucy**。
Lucy 是一個結合了現代「表達式化 (Expression-based)」設計與函數式「管道操作符 (Pipeline Operator)」的動態強型態程式語言，並包含一個對應的位元組碼編譯器與一個精簡高效的棧式虛擬機器 (LumiVM/LucyVM)。

---

## 🎯 系統程式概念與實作內容

* **詞法分析 (Lexical Analysis)**：手寫 Lexer，支援數字（浮點數）、字串字面量、關鍵字、識別碼以及雙字元操作符（如 `==`, `|>`）。
* **語法分析 (Syntax Analysis)**：手寫遞迴下降解析器 (Recursive Descent Parser)，將 Token 流轉化為抽象語法樹 (AST)，正確處理運算子優先級與 `if-else`、代碼塊的嵌套。
* **中間碼生成 (Bytecode Codegen)**：走訪 AST 並將其轉換為虛擬機器指令，編譯後的目的檔輸出為 **JSON 格式**（以 `.lucyc` 為副檔名），清晰展示常數表與指令序列。
* **虛擬機器 (Virtual Machine)**：棧式虛擬機器。具備獨立的 Call Frame 堆疊、區域變數作用域、全域變數表與操作數棧，支援遞迴調用。

---

## 📖 Lucy 語法規格 (EBNF)

```ebnf
program         = { statement }
statement       = variable_decl | function_decl | while_loop | expression_stmt
variable_decl   = "let" identifier "=" expression ";"
function_decl   = "fn" identifier "(" [ param_list ] ")" block
param_list      = identifier { "," identifier }
while_loop      = "while" expression block
expression_stmt = expression ";"

block           = "{" { statement } [ expression ] "}"

expression      = pipeline
pipeline        = comparison { "|>" identifier "(" [ arg_list ] ")" }
comparison      = additive [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) additive ]
additive        = multiplicative { ( "+" | "-" ) multiplicative }
multiplicative  = primary { ( "*" | "/" | "%" ) primary }
primary         = number | string | boolean | identifier | call | block | "(" expression ")" | if_expr
call            = identifier "(" [ arg_list ] ")"
arg_list        = expression { "," expression }
if_expr         = "if" expression block "else" block

number          = digit { digit }
string          = '"' { character } '"'
boolean         = "true" | "false"
identifier      = letter { letter | digit | "_" }
```

---

## ⚡ 虛擬機器指令集 (ISA)

本 VM 採用自訂的棧式虛擬機指令，支援 JSON 格式序列化：

* `LoadConst(idx)`：載入常數表中索引為 `idx` 的常數並推入棧頂。
* `LoadVar(name)`：尋找區域變數（或全域變數）`name`，推入棧頂。
* `StoreVar(name)`：彈出棧頂值，並存入目前作用域的變數 `name` 中。
* `Pop`：彈出並丟棄棧頂的值。
* `Add` / `Sub` / `Mul` / `Div` / `Mod`：算術運算。
* `Eq` / `Ne` / `Lt` / `Gt` / `Le` / `Ge`：關係運算。
* `Jump(pc)`：無條件跳躍到指定的指令索引。
* `JumpIfFalse(pc)`：彈出棧頂，若為假值 (false / nil) 則跳躍至指定索引。
* `Call(name, arg_count)`：建立新的 Call Frame，將 `arg_count` 個參數綁定至函數區域變數中，開始執行函數 `name`。
* `Return`：結束目前函數執行，回收 Call Frame。
* `Print`：彈出棧頂值並輸出至標準輸出。

---

## 🛠️ 編譯與執行說明

請確保您的電腦已安裝 Rust 工具鏈 (Cargo)。

進入 `HW02` 目錄：
```bash
cd HW02
```

### 1. 直接編譯並執行原始碼
```bash
cargo run -- run <path_to_file.lucy>
```

### 2. 僅編譯成位元組碼檔 (JSON 格式)
```bash
cargo run -- compile <path_to_file.lucy> <output_path.lucyc>
```
您可以直接開啟產生的 `.lucyc` 檔案，觀察內部的 JSON 結構（含 `constants`, `functions`, `instructions`）。

### 3. 使用虛擬機器執行位元組碼檔
```bash
cargo run -- exec <path_to_compiled.lucyc>
```

---

## 🚀 測試範例與示範

我們在專案中提供了三個範例檔：

### 1. 遞迴計算階乘 (`factorial.lucy`)
```rust
fn factorial(n) {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5);
print(result); // 輸出 120
```
**執行方式：**
```bash
cargo run -- run factorial.lucy
```

### 2. 管道操作符展示 (`pipeline.lucy`)
展示將左側的計算結果直接傳入右側函數的第一個參數中：
```rust
fn double(x) {
    x * 2
}

fn add_ten(x) {
    x + 10
}

// 管道傳遞計算： (5 * 2) + 10 = 20
let val = 5 |> double() |> add_ten();
print(val); // 輸出 20
```
**執行方式：**
```bash
cargo run -- run pipeline.lucy
```

### 3. 遞迴計算斐波那契數列 (`fibonacci.lucy`)
```rust
fn fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

let result = fib(10);
print(result); // 輸出 55
```
**執行方式：**
```bash
cargo run -- run fibonacci.lucy
```
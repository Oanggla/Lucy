# Lucy 程式語言與虛擬機器 (Rust 版) 實作成果報告

我們已成功使用 **Rust** 語言重新設計並實現了全新的 **Lucy** 程式語言編譯器與虛擬機器，並將其完整封裝在一個標準的 Cargo 專案中。

---

## 🏗️ 實作完成模組說明

本專案所有的程式碼與測試原始碼均存放於 [Lucy/](file:///c:/Users/angel/nqu/SystemProgramming/Lucy) 目錄下：

1. **詞法分析器 [src/lexer.rs](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/src/lexer.rs)**:
   - 負責將原始碼轉化成 `Token` 流。
   - 支援字串與數字解析、註解過濾以及雙字元符號處理（如 `|>` 與 `==`）。
2. **語法分析器 [src/parser.rs](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/src/parser.rs)**:
   - 遞迴下降解析器，建立 AST 結構。
   - 特色：在解析時，將表達式 `a |> func(b)` 自動降低（Lowering）為標準函數調用 `func(a, b)`，免去了虛擬機處理管道運算子的複雜指令。
3. **中間碼生成器 [src/codegen.rs](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/src/codegen.rs)**:
   - 遍歷 AST 生產與常數索引相關的虛擬機位元組碼。
   - 將編譯結果輸出為易讀的 **JSON 結構**，包含常數表（Constants）、函式（Functions）與主程式指令（Instructions）。
4. **棧式虛擬機器 [src/vm.rs](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/src/vm.rs)**:
   - 基於 CallFrame 與區域變數雜湊表的堆疊機器。
   - 精確處理跳躍分支指令，支援函數遞迴與內部變數作用域查找。
5. **主程式 [src/main.rs](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/src/main.rs)**:
   - 提供三個 CLI 子命令：`run` (直接執行原始碼)、`compile` (編譯為 JSON 位元組碼) 與 `exec` (執行 JSON 位元組碼)。
   - 底部附有 3 組涵蓋 Lexer、Parser、VM 以及管道運算的單元測試。

---

## 🧪 驗證與測試結果

### 1. 自動化單元測試 (`cargo test`)
我們編寫了 3 組測試，涵蓋 Lexer、Parser、Codegen、VM 與管道操作符，全部成功通過：
```bash
cargo test
```
**測試輸出：**
```text
running 3 tests
test tests::test_lexer ... ok
test tests::test_pipeline_operator ... ok
test tests::test_parser_and_vm ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. 遞迴階乘計算測試 (`factorial.lucy`)
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
**執行命令與結果：**
```bash
cargo run -- run factorial.lucy
```
- **輸出結果**：`120`

### 3. 管道操作符測試 (`pipeline.lucy`)
```rust
fn double(x) {
    x * 2
}

fn add_ten(x) {
    x + 10
}

let val = 5 |> double() |> add_ten();
print(val); // 輸出 20
```
**執行命令與結果：**
```bash
cargo run -- run pipeline.lucy
```
- **輸出結果**：`20`

### 4. 遞迴斐波那契數列測試 (`fibonacci.lucy`)
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
**執行命令與結果：**
```bash
cargo run -- run fibonacci.lucy
```
- **輸出結果**：`55`

---

## 🔍 編譯產生的目的檔 (JSON 格式)

當您執行以下指令：
```bash
cargo run -- compile factorial.lucy factorial.lucyc
```
會生成極其清晰的 [factorial.lucyc](file:///c:/Users/angel/nqu/SystemProgramming/Lucy/factorial.lucyc) 目的檔。這是一個易於助教與教授觀察評分的 JSON 位元組碼，結構如下：

```json
{
  "constants": [
    { "Number": 0.0 },
    { "Number": 1.0 },
    { "Number": 5.0 },
    "Nil"
  ],
  "functions": [
    {
      "name": "factorial",
      "params": [ "n" ],
      "instructions": [
        { "LoadVar": "n" },
        { "LoadConst": 0 },
        "Eq",
        { "JumpIfFalse": 6 },
        { "LoadConst": 1 },
        { "Jump": 12 },
        { "LoadVar": "n" },
        { "LoadVar": "n" },
        { "LoadConst": 1 },
        "Sub",
        { "Call": [ "factorial", 1 ] },
        "Mul",
        "Return"
      ]
    }
  ],
  "instructions": [
    { "LoadConst": 2 },
    { "Call": [ "factorial", 1 ] },
    { "StoreVar": "result" },
    { "LoadVar": "result" },
    "Print",
    { "LoadConst": 3 },
    "Pop"
  ]
}
```
隨後，您可以使用 `exec` 命令直接用虛擬機載入並運行此 JSON 檔：
```bash
cargo run -- exec factorial.lucyc
```
這會精準地輸出 `120`。
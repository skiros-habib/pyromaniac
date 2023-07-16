# Supported Languages

This document lists the details for each supported language in Pyromaniac. 

## Python
- **Version**: 3.11
- **3rd-party packages**: None

## Rust
- **Version**: latest stable
- **3rd-party crates**:
    - [rand (latest)](https://docs.rs/rand/latest/rand/)
    - [anyhow (latest)](https://docs.rs/anyhow/latest/anyhow/)
    - [itertools (latest)](https://docs.rs/itertools/latest/itertools/)
- **Compile mode**: [default `dev` profile](https://doc.rust-lang.org/cargo/reference/profiles.html#dev)

## Java
- **Version**: OpenJDK 17
- **Compile options**: none
- **Java Runtime options**: none
- **Caveats**: main class must be called `Main`, ie:

```java
public class Main{
    public static void main(String[] args){
        System.out.println("This is the main class");
    }
}
```

## Bash
GNU Bourne Again Shell, as included in https://hub.docker.com/_/bash
- **Version**: 5.2

## Sh (ash)
Busybox ash, as included in https://hub.docker.com/_/alpine
- **Version**: 3.18
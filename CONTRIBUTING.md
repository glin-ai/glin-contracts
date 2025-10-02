# Contributing to GLIN Smart Contracts

Thank you for your interest in contributing to GLIN! This document provides guidelines for contributing to our smart contract repository.

## 🤝 Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## 🚀 How to Contribute

### Reporting Bugs

1. **Check existing issues** to avoid duplicates
2. **Use the bug report template** when creating an issue
3. **Provide detailed information**:
   - Contract name and function
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment (ink! version, node version)

### Suggesting Enhancements

1. **Open an issue** with the enhancement template
2. **Describe the use case** and benefits
3. **Consider backwards compatibility**
4. **Discuss with maintainers** before implementing

### Pull Requests

1. **Fork the repository**
   ```bash
   git clone https://github.com/YOUR-USERNAME/glin-contracts.git
   cd glin-contracts
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow code style guidelines
   - Add tests for new functionality
   - Update documentation

4. **Test thoroughly**
   ```bash
   cargo test --workspace
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

5. **Commit with conventional commits**
   ```bash
   git commit -m "feat: add new escrow feature"
   git commit -m "fix: resolve overflow in vote calculation"
   git commit -m "docs: update registry README"
   ```

6. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a pull request on GitHub.

## 📝 Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

### Examples

```
feat(escrow): add multi-currency support

- Add currency parameter to create_agreement
- Update milestone resolution logic
- Add tests for multiple currencies

Closes #123
```

```
fix(registry): prevent stake overflow

The stake increase function could overflow when adding very large amounts.
Added checked arithmetic to prevent this.

Fixes #456
```

## 🧪 Testing Guidelines

### Unit Tests

Every contract must have unit tests covering:

- ✅ Happy path scenarios
- ✅ Error conditions
- ✅ Edge cases
- ✅ Access control
- ✅ Arithmetic operations

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn test_create_agreement_works() {
        // Arrange
        let mut contract = GenericEscrow::new(...);

        // Act
        let result = contract.create_agreement(...);

        // Assert
        assert!(result.is_ok());
    }

    #[ink::test]
    fn test_create_agreement_insufficient_funds() {
        // Test error conditions
    }
}
```

### Integration Tests

For complex interactions between contracts, add integration tests in the `integration-tests/` directory.

## 🎨 Code Style

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Maximum line length: 100 characters

### ink! Best Practices

1. **Use storage efficiently**
   ```rust
   // Good: Use Mapping for large datasets
   pub struct Contract {
       items: Mapping<u128, Item>,
   }

   // Bad: Don't use Vec for unbounded data
   pub struct Contract {
       items: Vec<Item>, // Can grow indefinitely
   }
   ```

2. **Validate inputs**
   ```rust
   #[ink(message)]
   pub fn process(&mut self, amount: Balance) -> Result<()> {
       if amount == 0 {
           return Err(Error::InvalidAmount);
       }
       // ... rest of logic
   }
   ```

3. **Emit events for important state changes**
   ```rust
   self.env().emit_event(AgreementCreated {
       agreement_id,
       client,
       provider,
   });
   ```

4. **Use Checks-Effects-Interactions pattern**
   ```rust
   // Check
   if caller != self.owner {
       return Err(Error::NotAuthorized);
   }

   // Effect
   self.balance -= amount;

   // Interaction
   self.env().transfer(recipient, amount)?;
   ```

## 📚 Documentation

### Code Documentation

Use Rust doc comments:

```rust
/// Creates a new escrow agreement with multiple milestones.
///
/// # Arguments
///
/// * `provider` - The service provider's account
/// * `milestones` - Vector of milestone descriptions
/// * `amounts` - Corresponding payment amounts
///
/// # Returns
///
/// Returns the agreement ID on success
///
/// # Errors
///
/// * `InsufficientFunds` - If transferred value < total amount
/// * `InvalidMilestoneCount` - If milestones and amounts lengths don't match
#[ink(message, payable)]
pub fn create_agreement(
    &mut self,
    provider: AccountId,
    milestones: Vec<String>,
    amounts: Vec<Balance>,
) -> Result<u128> {
    // Implementation
}
```

### README Updates

If your PR adds new functionality:

1. Update the contract's section in README.md
2. Add usage examples
3. Update gas cost estimates if applicable

## 🔒 Security Guidelines

### Before Submitting

- [ ] No private keys or secrets in code
- [ ] No unbounded loops or storage
- [ ] Arithmetic overflow protection
- [ ] Access control on sensitive functions
- [ ] Reentrancy protection where needed
- [ ] Gas optimization considered

### Security Checklist

```rust
// ✅ Good: Use checked arithmetic
let new_balance = balance.checked_add(amount)
    .ok_or(Error::Overflow)?;

// ❌ Bad: Can overflow
let new_balance = balance + amount;

// ✅ Good: Validate before state changes
if amount == 0 {
    return Err(Error::InvalidAmount);
}
self.balance += amount;

// ❌ Bad: No validation
self.balance += amount;
```

## 🏗️ Project Structure

```
glin-contracts/
├── escrow/           # GenericEscrow contract
├── registry/         # ProfessionalRegistry contract
├── arbitration/      # ArbitrationDAO contract
├── scripts/          # Build and deploy scripts
└── integration-tests/  # Cross-contract tests (coming soon)
```

## 🎯 Good First Issues

Look for issues labeled `good-first-issue` to get started:

- Documentation improvements
- Test coverage additions
- Gas optimizations
- Code cleanup

## 📧 Questions?

- **GitHub Discussions**: For general questions
- **Issues**: For bug reports and feature requests
- **Email**: dev@glin.network for private inquiries

## 📄 License

By contributing, you agree that your contributions will be licensed under Apache 2.0.

## 🙏 Recognition

All contributors will be recognized in our:
- CONTRIBUTORS.md file
- Release notes
- Annual reports

Thank you for making GLIN better! 🚀

pub mod backup;           // Public module for core backup functionality
mod backup_sets {         // Private parent module
    pub mod set_namer;    // Public submodule for naming backup sets
    pub mod backup_set;   // Public submodule for backup set operations
}
pub mod dhcopy;          // Public module for disk operations
pub mod test_helpers;    // Public module with shared test utilities
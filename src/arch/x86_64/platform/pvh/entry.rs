#[allow(bad_asm_style)]
core::arch::global_asm!(
	include_str!("entry.s"),
	rust_start = sym super::rust_start,
	stack = sym crate::arch::x86_64::stack::STACK,
	stack_top_offset = const crate::arch::x86_64::stack::Stack::top_offset(),
	level_4_table = sym crate::arch::x86_64::page_tables::LEVEL_4_TABLE,
	gdt_ptr = sym crate::arch::x86_64::gdt::GDT_PTR,
	kernel_code_selector = const crate::arch::x86_64::gdt::Gdt::kernel_code_selector().0,
	kernel_data_selector = const crate::arch::x86_64::gdt::Gdt::kernel_data_selector().0,
);

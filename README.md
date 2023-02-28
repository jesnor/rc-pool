# RcPool

## Overview

RcPool is a memory pool with items of the same type. It consists of a number of (minimum one) pages, each with a fixed number of item slots (not necessarily the same number). Item slots are automatically re-used in a memory safe way when possible. RcPool can be seen as a combination of/alternative to Rc/Weak, SlotMap and RefCell.


## Features

- Very fast, constant time allocation and free'ing (similar to SlotMap)
- Can optionally grow by allocating new pool pages from heap memory when full (but it will never shrink)
- Weak references are Copy which makes them easy and cheap to pass around and put in Cell's etc.
- Allows mutable access to an item with only one strong reference (similar to a RefCell)
- Configurable manual or automatic dropping of items
- Supports iteration over all live pool items
- Uses interior mutability so items can be inserted and removed through a shared pool reference (similar to how normal heap allocators works)


## Pool Capacity

When an RcPool is created it allocates one page with a fixed number of item slots. When inserting a new item there's an option to dynamically allocate a new slot page if there are no free slots in any of the current pages. The number of slots that newly allocated pages gets can be changed at any time.

Since there's no way of knowing how many weak references exists for slot in a page, a page can never be free'd and thus the pool capacity will never shrink. 


## Reference Types

RcPool supports two types of references:

- Weak references (similar to Weak and SlotMap's Key) consists of a shared reference to an item slot and the slot version number it expects in the slot. If the slot version number is different from the weak reference's, it cannot be upgraded to a strong reference. The size of a weak reference is two machine words (2x usize).

- Strong references (similar to Rc) which consists of a shared reference to an item slot. Strong references support obtaining a Rust shared reference to the item using the Deref trait and also a Rust mutable/unique reference if it's the only strong reference currently in existence (similar to RefCell). The size of strong reference is one machine word (usize).


## Item Dropping

Each RcPool can be configured in one of two drop modes:

- Automatic: similar to Weak/Rc in that the pool items are automatically dropped when the last strong reference is dropped

- Manual: similar to SlopMap in that items are only dropped explicitly. This can be useful if you want to only use weak references and use the pool itself as item owner.

Note that regardless of drop mode the dropped item memory can always be re-used even though there are weak references to the slot.


# Thread Safety

RcPool is not thread safe, so it can only be used from one thread, but you can of course create multiple pools in different threads. If you want to safely share items between threads instead use Arc with normal heap allocation. It's heavily optimized and safe for multithreaded usage. It doesn't make much sense to try to replace that.


## Comparison with Rc/Weak

Similarities:

- Support both weak and strong references
- The item can be automatically dropped when all strong references have been dropped

Differences:

- RcPool can also be configured to require manual dropping of items instead of automatic when all shared references are dropped

- Since RcPool uses versioning instead of a weak reference count it will re-use the slot/memory of a dropped item even if there are weak references to the slot. With Rc/Weak you have to drop all strong **and** weak references.

- RcPool's weak references are Copy which makes them easy and efficient to pass around and put in Cell's for example
- Weak is only one machine word in size, while RcPool's weak references are two machine words
- You can obtain a mutable reference to an RcPool item with one strong reference even if there are weak references to it. This is similar to a RefCell.
- RcPool allocation and free'ing is much faster than using the system allocator like Rc does (TODO: benchmark)
- An RcPool will never free any allocated heap memory unless the entire pool is dropped


## Comparison with SlotMap

Similarities:

- RcPool's weak references and SlotMap keys are similar in that they consists of a version number and a shared index/reference
- Like SlotMap keys, RcPool's weak references implements Copy
- Allows iteration over all live pool items
- Supports manual dropping of items (when there's zero or one strong reference)
- Very fast, constant time allocation and free'ing of items (when not needing to allocate more item slots)

Differences:

- SlotMap needs to copy all items when allocating new space, while RC pool simply allocates a new empty page of customizable size

- RcPool references contains direct shared references to the item slot so there's no need to pass around a reference to the pool to be able to dereference them. This makes them both safer and more convenient to use.

- RcPool supports strong references with automatic dropping of items, SlotMap doesn't have strong references

- SlotMap keys are smaller as they consists of a 32-bit index and version number (total of 8 bytes), while RcPool's weak references consists of a shared reference and a 64-bit version number (16 bytes on a 64-bit machine). However, this is not just a downside as using a 64-bit version number totally eliminates the risk of version collisions (at most once every 584 years with 1 billion updates per second), and RcPool supports larger pools (SlotMap is limited to 2^32 items).

- RcPool uses interior mutability so items can be inserted and removed through a shared pool reference. SlotMap requires a mutable reference.

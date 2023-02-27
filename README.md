# RcPool

## Overview

RcPool is a single threaded memory pool with items of the same type. It consists of a number of (minimum one) fixed size pages with item slots. Slots are automatically re-used in a safe way when possible. It can be seen as a combination of/replacement for Rc/Weak, SlotMap and RefCell.


## Features

- Very fast, constant time allocation and free'ing (similar to SlotMap)
- Can optionally grow by allocating new pool pages from heap memory when full (currently it never shrinks though)
- Weak references are Copy which makes them easy to pass around and put in Cell's etc.
- Allows mutable access to an item with only one strong reference (similar to a RefCell)
- Configurable manual or automatic dropping of items
- Supports iteration over all live pool items


## Pool Capacity

The RcPool capacity can optionally grow automatically when new items are added and there are no free slots available. This is customizable for every addition of a new item. The growth amount is also customizable. Currently the RcPool capacity will never shrink (this might change in the future).


## Reference Types

RcPool supports two type of references:

- Weak references (similar to Weak and SlotMap's Key) consists of a shared reference to an item slot and the slot version number it expects in the slot. If the slot version number is different from the weak reference's, it cannot be upgraded to a strong reference. The size of a weak reference is two machine words (2x usize).

- Strong references (similar to Rc) which consists of a shared reference to an item slot. Strong references support obtaining a Rust shared reference to the item and also a Rust mutable/unique reference if it's the only strong reference currently in existence (similar to RefCell). The size of strong reference is one machine word (usize).


## Item Dropping

Each RcPool can be configured in one of two drop modes:

- Automatic: similar to Weak/Rc in that the pool items are automatically dropped when the last strong reference is dropped

- Manual: similar to SlopMap in that items are only dropped explicitly. This can be useful if you want to only use weak references and use the pool itself as item owner.

Note that regardless of drop mode the dropped item memory can always be re-used even though there are weak references remaining. 


## Comparison with Rc/Weak

Similarities:

- Support both weak and strong references
- The item can be automatically dropped when all strong references have been dropped

Differences:

- RcPool can also be configured to require manual dropping of items instead of automatic

- Since RcPool uses versioning instead of a weak reference count it will re-use the memory of a dropped item even if there are weak references remaining. With Rc/Weak you have to drop all strong *and* weak references.

- RcPool's weak references are Copy which makes them easier to pass around and put in Cell's for example
- Weak is only one machine word in size, but RcPool's weak references are two machine words
- You can obtain a mutable reference to a RcPool item with one strong reference even if there are weak references to it. This enables using it like a RefCell.
- RcPool allocation and free'ing is much faster than using the system allocator like Rc does (TODO: benchmark)


## Comparison vs SlotMap

Similarities:

- RcPool's weak references and SlotMap keys are similar in that they consists of a version number and a shared index/reference, and they both implement Copy
- Allows iteration over all live pool items
- Supports manual dropping of items (SlotMap doesn't have strong references so there can't be any automatic dropping)
- Very fast, constant time allocation and free'ing of items (when not space is not growing)

Differences:

- SlotMap needs to copy all items when allocating new space, while RC pool simply allocates a new empty page of customizable size

- RcPool references contains direct shared references to the item slot so there's no need to pass around the pool to be able to dereference them. This makes them both safer and more convenient to use.

- RcPool supports strong references with automatic dropping of items, something SlotMap doesn't have

- SlotMap keys uses 32-bit version and index, while RC pool's weak references are two machine words (16 bytes on a 64-bit machine). However, one advantage is that a 64-bit version number totally eliminates the risk of version collisions (at most once every 584 years with 1 billion updates per second), and RcPool supports larger pools (2^64 compared to 2^32 items).

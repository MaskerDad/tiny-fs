# tiny-fs
> a tiny filesystem from xv6/rCore

常规文件和目录都是实际保存在持久存储设备中的。持久存储设备仅支持以扇区（或块）为单位的随机读写，这和上面介绍的通过路径即可索引到文件并以字节流进行读写的用户视角有很大的不同。负责中间转换的便是 **文件系统** (File System) 。具体而言，文件系统负责将逻辑上的目录树结构（包括其中每个文件或目录的数据和其他信息）映射到持久存储设备上，决定设备上的每个扇区应存储哪些内容。反过来，文件系统也可以从持久存储设备还原出逻辑上的目录树结构。

在一个计算机系统中，可以同时包含多个持久存储设备，它们上面的数据可能是以不同文件系统格式存储的。为了能够对它们进行统一管理，在内核中有一层 **虚拟文件系统** (VFS, Virtual File System) ，它规定了逻辑上目录树结构的通用格式及相关操作的抽象接口，只要不同的底层文件系统均实现虚拟文件系统要求的那些抽象接口，再加上 **挂载** (Mount) 等方式，这些持久存储设备上的不同文件系统便可以用一个统一的逻辑目录树结构一并进行管理。

> 松耦合模块化设计：与操作系统内核完全解耦

* 与底层设备驱动之间通过抽象接口 `BlockDevice` 来连接，避免了与设备驱动的绑定；
* 通过Rust提供的alloc crate来隔离了操作系统内核的内存管理，避免了直接调用内存管理的内核函数；
* 在底层驱动上，采用的是轮询的方式访问 `virtio_blk` 虚拟磁盘设备，从而避免了访问外设中断的相关内核函数；
* 避免了直接访问进程相关的数据和函数，从而隔离了操作系统内核的进程管理。

---

- 扁平化：仅存在根目录 `/` 一个目录，剩下所有的文件都放在根目录内。在索引一个文件的时候，我们直接使用文件的文件名而不是它含有 `/` 的绝对路径。
- 权限控制：我们不设置用户和用户组概念，全程只有单用户。同时根目录和其他文件也都没有权限控制位，即完全不限制文件的访问方式，不会区分文件是否可执行。
- 不记录文件访问/修改的任何时间戳。
- 不支持软硬链接。
- 除了下面即将介绍的系统调用之外，其他的很多文件系统相关系统调用均未实现。

![简化的文件和目录示意图](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/simple-file-and-dir.png)

# content

`tiny-fs` 文件系统的整体架构自下而上可分为五层：

1. **磁盘块设备接口层：**定义了以块大小为单位对磁盘块设备进行读写的trait接口
2. **块缓存层：**在内存中缓存磁盘块的数据，避免频繁读写磁盘
3. **磁盘数据结构层**：磁盘上的超级块、位图、索引节点、数据块、目录项等核心数据结构和相关处理
4. **磁盘块管理器层（PFS）：**合并了上述核心数据结构和磁盘布局所形成的磁盘文件系统数据结构，以及基于这些结构的创建/打开文件系统的相关处理和磁盘块的分配和回收处理
5. **索引节点层（VFS）：**管理索引节点（即文件控制块）数据结构，并实现文件创建/文件打开/文件读写等成员函数来向上支持文件操作相关的系统调用

作为一个文件系统而言，它的磁盘布局体现在磁盘上各扇区的内容上，而它解析磁盘布局得到的逻辑目录树结构则是通过内存上的数据结构来访问的，这意味着它要同时涉及到对磁盘和对内存的访问。它们的访问方式是不同的，对于内存直接通过一条指令即可直接读写内存相应的位置，而磁盘的话需要用软件的方式向磁盘发出请求来间接进行读写。

## BlockDevice

> Introduction

`BlockDevice` trait 代表了一个抽象块设备的接口，该 trait 仅需求两个函数，数据需要以块为单位进行读写：

*  `read_block` : 将数据从块设备读到内存缓冲区中
* `write_block` : 将数据从内存缓冲区写回到块设备中

tiny-fs 库的使用者（如操作系统内核）需要实现块设备驱动程序，并实现 `BlockDevice` trait 以提供给 tiny-fs 库使用，这样 tiny-fs 库就与一个具体的执行环境对接起来了。

## BlockCache

操作系统的最底层（即块设备驱动程序）已经有了对块设备的读写能力，但从编程方便/正确性和读写性能的角度来看，仅有块读写这么基础的底层接口是不足以实现高效的文件系统，为什么？

* 某应用将一个块的内容读到内存缓冲区，对缓冲区进行修改，并尚未写回块设备时，如果另外一个应用再次将该块的内容读到另一个缓冲区，而不是使用已有的缓冲区，这将会造成数据不一致；
* 可能增加很多不必要的块读写次数，大幅降低文件系统的性能；

> 如何设计？

* `BlockCache` 代表一个被我们管理起来的块缓冲区，它包含块数据内容以及块的编号等信息。当它被创建的时候，将触发一次 `read_block` 将数据从块设备读到它的缓冲区中。接下来只要它驻留在内存中，便可保证对于同一个块的所有操作都会直接在它的缓冲区中进行而无需额外的 `read_block` ；
* 块缓存管理器 `BlockManager` 在内存中管理有限个 `BlockCache` 并实现了类似 FIFO 的缓存替换算法，当一个块缓存被换出的时候视情况可能调用 `write_block` 将缓冲区数据写回块设备。总之，块缓存层对上提供 `get_block_cache` 接口来屏蔽掉相关细节，从而可以向上层子模块提供透明读写数据块的服务。

## DiskLayout

tiny-fs文件系统中的所有需要持久保存的数据都会放到磁盘上，这包括了管理这个文件系统的 **超级块 (Super Block)**，管理空闲磁盘块的 **索引节点位图区** 和 **数据块位图区** ，以及管理文件的 **索引节点区** 和 放置文件数据的 **数据块区** 组成。

![../_images/文件系统布局.png](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/%E6%96%87%E4%BB%B6%E7%B3%BB%E7%BB%9F%E5%B8%83%E5%B1%80.png)

## DiskManager (PFS)

tiny-fs文件系统中管理这些磁盘数据的控制逻辑主要集中在 **磁盘块管理器** 中，其核心是 `DiskManager` 数据结构及其关键成员函数：

- EasyFileSystem.create：创建文件系统
- EasyFileSystem.open：打开文件系统
- EasyFileSystem.alloc_inode：分配inode （dealloc_inode未实现，所以还不能删除文件）
- EasyFileSystem.alloc_data：分配数据块
- EasyFileSystem.dealloc_data：回收数据块

## VirtualFileSystem (VFS)

对于单个文件的管理和读写的控制逻辑主要是 **索引节点（文件控制块）** 来完成，其核心是 `Inode` 数据结构及其关键成员函数：

- Inode.new：在磁盘上的文件系统中创建一个inode
- Inode.find：根据文件名查找对应的磁盘上的inode
- Inode.create：在根目录下创建一个文件
- Inode.read_at：根据inode找到文件数据所在的磁盘数据块，并读到内存中
- Inode.write_at：根据inode找到文件数据所在的磁盘数据块，把内存中数据写入到磁盘数据块中

---

# core design







# step-by-step

RBE

* 闭包
* queue

---

- [ ] `BlockDevice`

  在 `tiny-fs` 库的最底层声明了一个块设备的抽象接口 `BlockDevice`，其包含两个方法 `read_block/write_block` 

- [ ] `BlockCache`

  - [ ] `struct BlockCache`：由于操作系统频繁读写速度缓慢的磁盘块会极大降低系统性能，因此常见的手段是先将一个块上的数据从磁盘读到内存中的一个缓冲区中；
    - [ ] 创建一个 `BlockCache` 的时候，这将触发一次 `read_block` 将一个块上的数据从磁盘读到缓冲区；
    - [ ] `BlockCache` 的设计也体现了 RAII 思想， 它管理着一个缓冲区的生命周期。当 `BlockCache` 的生命周期结束之后缓冲区也会被从内存中回收，这个时候 `modified` 标记将会决定数据是否需要写回磁盘；
    - [ ] `BlockCache::read/modify` 让上层操作块缓存更加方便；
  - [ ] `struct BlockCacheManager`：为了避免在块缓存上浪费过多内存，我们希望内存中同时只能驻留有限个磁盘块的缓冲区。
    - [ ] `get_block_cache` 方法尝试从块缓存管理器中获取一个编号为 `block_id` 的块的块缓存，如果找不到，会从磁盘读取到内存中，还有可能会发生缓存替换；
    - [ ] 创建 `BlockCacheManager` 的全局实例；

- [ ] `DiskLayout`

  - [ ] `SuperBlock`：存放在磁盘上编号为 0 的块的起始处
    - [ ] `initialize` 创建一个 tiny-fs 时对超级块进行初始化；
    - [ ] `is_valid` 通过魔数判断超级块所在的文件系统是否合法；
  - [ ] `Inode/Data_BitMap`
    - [ ] 位图 `Bitmap` 中仅保存了它所在区域的起始块编号以及区域的长度为多少个块；
    - [ ]  `Bitmap::alloc/dealloc` 通过置位/清零 bit 来分配/回收磁盘块 => 返回全局bit
  - [ ] `Inode`
    - [ ] 每个文件/目录在磁盘上均以一个 `DiskInode` 的形式存储；
    - [ ] 索引方式分成直接索引和间接索引两种；
    - [ ] `get_block_id` 可以从索引中查到它自身用于保存文件内容的第 `block_id` 个数据块的块编号 => 返回全局块ID
    - [ ] 在对文件/目录初始化之后，它的 `size` 均为 0 ，需要通过 `increase_size` 方法逐步扩充容量。
      - [ ] `block_num_needed` 确定在容量扩充的时候额外需要多少块；
      - [ ] `increase_size` 接收两个参数，其中 `new_size` 表示容量扩充之后的文件大小， `new_blocks` 是一个保存了本次容量扩充所需块编号的向量，这些块都是由上层的磁盘块管理器负责分配的；
    - [ ] 通过 `clear_size` 方法实现清空文件的内容并回收所有数据和索引块，将回收的所有块的编号保存在一个向量中返回给磁盘块管理器；
    - [ ] `DiskInode::read_at/write_at` 来读写它索引的那些数据块中的数据；
  - [ ] `Data/DirEntry`
    - [ ] 作为一个文件而言，每个保存内容的数据块都只是一个字节数组 `[u8; BLOCK_SZ]`；
    - [ ] 目录项是一个二元组，包括两个元素：文件名/子目录名以及文件（或子目录）所在的索引节点编号；

- [ ] `DiskManager (pfs)`

  - [ ] `EasyFileSystem` 包含两个位图 `inode_bitmap` 和 `data_bitmap` ，还记录下索引节点区域和数据块区域起始块编号 `inode_area_start_block/data_area_start_block`；
  - [ ] 通过 `create` 方法可以在块设备上创建并初始化一个 tiny-fs；
  - [ ] 通过 `open` 方法可以从一个已写入了 tiny-fs 镜像的块设备上打开 tiny-fs； 

- [ ] `INode (vfs)`：`DiskInode` 放在磁盘块中比较固定的位置，而 `Inode` 是放在内存中的记录文件索引节点信息的数据结构。

  - [ ] `INode` 的相关操作需要通过  `DiskManager` 访问底层的 `DiskINode`，进而获取到真正的数据；
  - [ ] 设计两个方法 `read_disk_inode/modify_disk_inode` 简化对于 `INode` 对应的磁盘上 `DiskInode` 的访问流程；
  - [ ] 获取根目录的 `INode`
  - [ ] 文件索引

  ---

  //TODO

  - [ ] 文件列举
  - [ ] 文件创建
  - [ ] 文件清空
  - [ ] 文件读写




# test

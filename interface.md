# Interface

lwext4C语言导出的接口描述

## lwext4

`ext4_dmask_set`: Global mask debug set.

`ext4_device_register`: Register block device.

`ext4_mount`: Mount a block device with EXT4 partition to the mount point.

`ext4_recover`: Journal recovery.

`ext4_journal_start`: Starts journaling. Journaling start/stop functions are transparent and might be used on filesystems without journaling support.

`ext4_cache_write_back`: Enable/disable write back cache mode.

`ext4_journal_stop`: Stops journaling. Journaling start/stop functions are transparent and might be used on filesystems without journaling support.

`ext4_umount`: Umount operation.

`ext4_mount_point_stats`: Get file mount point stats.

### File

`ext4_fremove`: Remove file by path

`ext4_fopen`: File open function.

`ext4_fopen2`: Alternate file open function

`ext4_fwrite`: Write data to file.

`ext4_fclose`: File close function.

- [ ] `ext4_frename`: Rename file

### Dir

`ext4_dir_open`: Directory open.

`ext4_dir_rm`: Recursive directory remove.

`ext4_dir_entry_next`: Return next directory entry.

`ext4_dir_close`: Directory close

`ext4_dir_mk`:  Create new directory.



## Inode

`ext4_raw_inode_fill`: Get inode of file/directory/link.

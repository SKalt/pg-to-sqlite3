-- define a normalized database that defines a filesystem
CREATE TABLE _blob (
  -- TODO: rename to blobs to avoid type-keyword
  id BIGSERIAL PRIMARY KEY,
  n_bytes INT,
  sha256 CHAR(64) UNIQUE NOT NULL
);
-- COMMENT ON _blob IS ''
CREATE TABLE _directory (
  id SERIAL PRIMARY KEY,
  absolute_path TEXT UNIQUE NOT NULL
);
CREATE TABLE _file_name(id SERIAL PRIMARY KEY, name TEXT UNIQUE);
CREATE TABLE _file (
  -- plural to avoid keywords
  id SERIAL PRIMARY KEY,
  directory_id INT REFERENCES _directory(id),
  file_name_id INT REFERENCES _file_name(id),
  mode BIT(32),
  blob_id INT REFERENCES _blob(id),
  CONSTRAINT unique_file_info UNIQUE(
    directory_id,
    file_name_id,
    mode,
    blob_id
  )
);
-- this reads golang's representation of file modes, NOT the POSIX standard file modes.
-- https://golang.org/pkg/os/#FileMode
-- https://golang.org/src/os/types.go?s=2790:2823#L55
-- https://github.com/Peltoche/lsd/blob/master/src/meta/permissions.rs
-- https://endler.dev/2018/ls/#implementing-very-basic-file-mode
CREATE VIEW files AS
SELECT f.id,
  _directory.absolute_path AS directory,
  _file_name.name,
  _blob.sha256,
  _blob.n_bytes,
  f.mode AS mode_bits,
  (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 31) = (CAST(1 AS bit(32)) << 31) THEN 'd' -- is a directory
      WHEN f.mode & (CAST(1 AS bit(32)) << 30) = (CAST(1 AS bit(32)) << 30) THEN 'a' -- append-only
      WHEN f.mode & (CAST(1 AS bit(32)) << 29) = (CAST(1 AS bit(32)) << 29) THEN 'l' -- exclusive use
      WHEN f.mode & (CAST(1 AS bit(32)) << 28) = (CAST(1 AS bit(32)) << 28) THEN 'T' -- temporary file; Plan 9 only
      WHEN f.mode & (CAST(1 AS bit(32)) << 27) = (CAST(1 AS bit(32)) << 27) THEN 'L' -- symbolic link
      WHEN f.mode & (CAST(1 AS bit(32)) << 26) = (CAST(1 AS bit(32)) << 26) THEN 'D' -- device file
      WHEN f.mode & (CAST(1 AS bit(32)) << 25) = (CAST(1 AS bit(32)) << 25) THEN 'p' -- named pipe (FIFO)
      WHEN f.mode & (CAST(1 AS bit(32)) << 24) = (CAST(1 AS bit(32)) << 24) THEN 'S' -- Unix domain socket
      WHEN f.mode & (CAST(1 AS bit(32)) << 23) = (CAST(1 AS bit(32)) << 23) THEN 'u' -- setuid
      WHEN f.mode & (CAST(1 AS bit(32)) << 22) = (CAST(1 AS bit(32)) << 22) THEN 'g' -- setgid
      WHEN f.mode & (CAST(1 AS bit(32)) << 21) = (CAST(1 AS bit(32)) << 21) THEN 'c' -- Unix character device, when ModeDevice is set
      WHEN f.mode & (CAST(1 AS bit(32)) << 20) = (CAST(1 AS bit(32)) << 20) THEN 't' -- sticky
      WHEN f.mode & (CAST(1 AS bit(32)) << 19) = (CAST(1 AS bit(32)) << 19) THEN '?' -- non-regular file; nothing else is known about this file
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 8) = (CAST(1 AS bit(32)) << 8) THEN 'r'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 7) = (CAST(1 AS bit(32)) << 7) THEN 'w'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 6) = (CAST(1 AS bit(32)) << 6) THEN 'x'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 5) = (CAST(1 AS bit(32)) << 5) THEN 'r'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 4) = (CAST(1 AS bit(32)) << 4) THEN 'w'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 3) = (CAST(1 AS bit(32)) << 3) THEN 'x'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode &(CAST(1 AS bit(32)) << 2) = (CAST(1 AS bit(32)) << 2) THEN 'r'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 1) = (CAST(1 AS bit(32)) << 1) THEN 'w'
      ELSE '-'
    END
  ) || (
    CASE
      WHEN f.mode & (CAST(1 AS bit(32)) << 0) = (CAST(1 AS bit(32)) << 0) THEN 'x'
      ELSE '-'
    END
  ) AS mode_string,
  f.mode & (CAST(1 AS bit(32)) << 31) = (CAST(1 AS bit(32)) << 31) AS mode_dir,
  -- d: is a directory
  f.mode & (CAST(1 AS bit(32)) << 30) = (CAST(1 AS bit(32)) << 30) AS mode_append,
  -- a: append-only
  f.mode & (CAST(1 AS bit(32)) << 29) = (CAST(1 AS bit(32)) << 29) AS mode_exclusive,
  -- l: exclusive use
  f.mode & (CAST(1 AS bit(32)) << 28) = (CAST(1 AS bit(32)) << 28) AS mode_temporary,
  -- T: temporary file; Plan 9 only
  f.mode & (CAST(1 AS bit(32)) << 27) = (CAST(1 AS bit(32)) << 27) AS mode_symlink,
  -- L: symbolic link
  f.mode & (CAST(1 AS bit(32)) << 26) = (CAST(1 AS bit(32)) << 26) AS mode_device,
  -- D: device file
  f.mode & (CAST(1 AS bit(32)) << 25) = (CAST(1 AS bit(32)) << 25) AS mode_namedPipe,
  -- p: named pipe (FIFO)
  f.mode & (CAST(1 AS bit(32)) << 24) = (CAST(1 AS bit(32)) << 24) AS mode_socket,
  -- S: Unix domain socket
  f.mode & (CAST(1 AS bit(32)) << 23) = (CAST(1 AS bit(32)) << 23) AS mode_setuid,
  -- u: setuid
  f.mode & (CAST(1 AS bit(32)) << 22) = (CAST(1 AS bit(32)) << 22) AS mode_setgid,
  -- g: setgid
  f.mode & (CAST(1 AS bit(32)) << 21) = (CAST(1 AS bit(32)) << 21) AS mode_char_device,
  -- c: Unix character device, when ModeDevice is set
  f.mode & (CAST(1 AS bit(32)) << 20) = (CAST(1 AS bit(32)) << 20) AS mode_sticky,
  -- t: sticky
  f.mode & (CAST(1 AS bit(32)) << 19) = (CAST(1 AS bit(32)) << 19) AS mode_irregular,
  -- ?: non-regular file; nothing else is known about this file
  f.mode & (CAST(1 AS bit(32)) << 8) = (CAST(1 AS bit(32)) << 8) AS user_read,
  f.mode & (CAST(1 AS bit(32)) << 7) = (CAST(1 AS bit(32)) << 7) AS user_write,
  f.mode & (CAST(1 AS bit(32)) << 6) = (CAST(1 AS bit(32)) << 6) AS user_execute,
  f.mode & (CAST(1 AS bit(32)) << 5) = (CAST(1 AS bit(32)) << 5) AS group_read,
  f.mode & (CAST(1 AS bit(32)) << 4) = (CAST(1 AS bit(32)) << 4) AS group_write,
  f.mode & (CAST(1 AS bit(32)) << 3) = (CAST(1 AS bit(32)) << 3) AS group_execute,
  f.mode & (CAST(1 AS bit(32)) << 2) = (CAST(1 AS bit(32)) << 2) AS other_read,
  f.mode & (CAST(1 AS bit(32)) << 1) = (CAST(1 AS bit(32)) << 1) AS other_write,
  f.mode & (CAST(1 AS bit(32)) << 0) = (CAST(1 AS bit(32)) << 0) AS other_execute
FROM _file AS f
  JOIN _directory ON f.directory_id = _directory.id
  JOIN _blob ON f.blob_id = _blob.id
  JOIN _file_name ON f.file_name_id = _file_name.id;

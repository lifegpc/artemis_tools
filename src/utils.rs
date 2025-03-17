use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

/// 查找指定目录下的所有.ast文件
///
/// # 参数
///
/// * `path` - 要搜索的目录路径
/// * `recursive` - 是否递归搜索子目录
///
/// # 返回
///
/// 包含所有找到的.ast文件路径的字符串向量
pub fn find_ast_files(path: &String, recursive: bool) -> io::Result<Vec<String>> {
    let mut result = Vec::new();
    let dir_path = Path::new(&path);

    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "ast") {
                if let Some(path_str) = path.to_str() {
                    result.push(path_str.to_string());
                }
            } else if recursive && path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    let mut sub_files = find_ast_files(&path_str.to_string(), recursive)?;
                    result.append(&mut sub_files);
                }
            }
        }
    }

    Ok(result)
}

/// 收集所有的.ast文件
///
/// # 参数
///
/// * `files` - 文件或目录路径列表。如果为空，则使用当前工作目录
/// * `recursive` - 是否递归搜索子目录
///
/// # 返回
///
/// 包含所有找到的.ast文件路径的字符串向量
pub fn collect_ast_files(files: &Vec<String>, recursive: bool) -> io::Result<Vec<String>> {
    let mut result = Vec::new();

    // 如果files为空，使用当前工作目录
    if files.is_empty() {
        let cwd = env::current_dir()?;
        if let Some(cwd_str) = cwd.to_str() {
            let mut cwd_files = find_ast_files(&cwd_str.to_string(), recursive)?;
            result.append(&mut cwd_files);
        }
    } else {
        // 处理files列表中的每个文件或目录
        for file in files {
            if file == "-" {
                result.push("-".into());
                continue;
            }
            let path = PathBuf::from(&file);

            if path.is_file() {
                // 如果是.ast文件，直接添加
                if path.extension().map_or(false, |ext| ext == "ast") {
                    result.push(file.clone());
                }
            } else if path.is_dir() {
                // 如果是目录，调用find_ast_files遍历
                let mut dir_files = find_ast_files(file, recursive)?;
                result.append(&mut dir_files);
            }
        }
    }

    Ok(result)
}

pub fn read_file<F: AsRef<Path> + ?Sized>(f: &F) -> io::Result<Vec<u8>> {
    let mut content = Vec::new();
    if f.as_ref() == Path::new("-") {
        io::stdin().read_to_end(&mut content)?;
    } else {
        content = fs::read(f)?;
    }
    Ok(content)
}

pub fn write_file<F: AsRef<Path> + ?Sized>(f: &F) -> io::Result<Box<dyn Write>> {
    Ok(if f.as_ref() == Path::new("-") {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create(f)?)
    })
}

INSERT OR IGNORE INTO file_type_blacklist (extension, description, is_default, created_at)
VALUES
  ('.jse', '编码后的 JavaScript 脚本', 1, strftime('%s','now')),
  ('.vbe', '编码后的 VBScript 脚本', 1, strftime('%s','now')),
  ('.vb', 'VBScript 文件变体', 1, strftime('%s','now')),
  ('.psm1', 'PowerShell 模块文件', 1, strftime('%s','now')),
  ('.psd1', 'PowerShell 模块数据文件', 1, strftime('%s','now')),
  ('.cpl', '控制面板扩展项，本质是 DLL', 1, strftime('%s','now')),
  ('.msp', 'Windows Installer 补丁', 1, strftime('%s','now')),
  ('.mst', 'Windows Installer 转换文件', 1, strftime('%s','now')),
  ('.appref-ms', 'ClickOnce 部署引用', 1, strftime('%s','now')),
  ('.docm', '含宏的 Word 文档', 1, strftime('%s','now')),
  ('.xlsm', '含宏的 Excel 文档', 1, strftime('%s','now')),
  ('.pptm', '含宏的 PowerPoint 文档', 1, strftime('%s','now')),
  ('.dotm', '含宏的 Word 模板', 1, strftime('%s','now')),
  ('.pl', 'Perl 脚本', 1, strftime('%s','now')),
  ('.rb', 'Ruby 脚本', 1, strftime('%s','now')),
  ('.php', 'PHP 脚本', 1, strftime('%s','now')),
  ('.pyc', 'Python 编译字节码', 1, strftime('%s','now')),
  ('.gadget', 'Windows 桌面小工具', 1, strftime('%s','now')),
  ('.scr', '屏幕保护程序，本质是 exe', 1, strftime('%s','now')),
  ('.msi', 'Windows Installer 安装包', 1, strftime('%s','now')),
  ('.ps1', 'PowerShell 脚本', 1, strftime('%s','now')),
  ('.vbs', 'VBScript 脚本', 1, strftime('%s','now')),
  ('.js', 'JavaScript（WSH 宿主环境）', 1, strftime('%s','now')),
  ('.bat', '批处理文件', 1, strftime('%s','now')),
  ('.cmd', 'Windows 命令脚本', 1, strftime('%s','now')),
  ('.pif', '程序信息文件', 1, strftime('%s','now')),
  ('.com', 'DOS 可执行文件', 1, strftime('%s','now')),
  ('.wsf', 'Windows Script File', 1, strftime('%s','now')),
  ('.hta', 'HTML Application', 1, strftime('%s','now')),
  ('.jar', 'Java 程序', 1, strftime('%s','now')),
  ('.lnk', '快捷方式', 1, strftime('%s','now')),
  ('.reg', '注册表文件', 1, strftime('%s','now')),
  ('.sh', 'Shell 脚本', 1, strftime('%s','now')),
  ('.bin', '二进制可执行文件', 1, strftime('%s','now')),
  ('.run', '自解压安装脚本', 1, strftime('%s','now')),
  ('.appimage', 'Linux 应用打包格式', 1, strftime('%s','now')),
  ('.py', 'Python 脚本', 1, strftime('%s','now')),
  ('.msc', 'Microsoft 管理控制台单元', 1, strftime('%s','now'));

INSERT OR IGNORE INTO file_access_policy (policy_key, enabled, updated_at)
VALUES
  ('exec_control', 0, strftime('%s','now')),
  ('auto_read_control', 0, strftime('%s','now')),
  ('file_type_blacklist_control', 0, strftime('%s','now'));

INSERT OR IGNORE INTO exec_type (type_name, description)
VALUES
  ('dll', '动态链接库'),
  ('exe', 'Windows 可执行文件'),
  ('PE', 'Windows PE 可执行文件格式'),
  ('ELF', 'Linux 原生可执行文件格式');

INSERT OR IGNORE INTO system_config (config_key, config_value, updated_at)
VALUES
  ('device_description', '(AD USB protection dev)USB Device', strftime('%s','now')),
  ('auth_status', 'unauthorized', strftime('%s','now')),
  ('auth_expire_time', '0', strftime('%s','now')),
  ('machine_code', '', strftime('%s','now')),
  ('system_version', '1.0.0', strftime('%s','now')),
  ('virus_db_package_version', 'v0.0.0', strftime('%s','now')),
  ('virus_db_version', '', strftime('%s','now')),
  ('virus_db_updated_at', '0', strftime('%s','now'));

INSERT OR IGNORE INTO users
  (username, password_hash, role, status, is_builtin, login_fail_count, created_at, updated_at)
VALUES
  ('admin', '$2b$12$ZDhWMHU7IE.y3Bwj8iRmrekwJT52DQxDx33mVNz3hbLCZ9g5/NLwO', 0, 0, 1, 0, strftime('%s','now'), strftime('%s','now')),
  ('operator', '$2b$12$nC77GZiBjNtullz9Zu4YUuDQ0XYMXKxcLWLkrYDDUYQKxBnaVj7X2', 1, 0, 1, 0, strftime('%s','now'), strftime('%s','now')),
  ('audit', '$2b$12$07S1.9Mzmw26mMpp4zcx.O3gdFN/lMSmOpKo3xG01e2c4R52KW/HK', 2, 0, 1, 0, strftime('%s','now'), strftime('%s','now'));

INSERT OR IGNORE INTO role_permission (role, page_key)
VALUES
  (0, 'system_management'),
  (0, 'user_management'),
  (1, 'file_access_control'),
  (1, 'usb_device_control'),
  (1, 'policy_management'),
  (2, 'log_management');

vim.keymap.set("n", "<leader>cr", function()
    vim.cmd("ToggleTerm")
    -- Send the command to the default terminal
    vim.cmd("TermExec cmd='cargo run -- text poem.txt'")
end)

local dap = require("dap")

dap.adapters.lldb = {
    type = "executable",
    command = "/usr/bin/lldb-dap",
    name = "lldb",
}

dap.configurations.rust = {
    {
        name = "mygrep",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/mygrep"
        end,
        cwd = "${workspaceFolder}",
        runInTerminal = false,
        stopOnEntry = false,
        args = {"text", "poem.txt"},
    },
}

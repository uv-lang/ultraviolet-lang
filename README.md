# Ultraviolet

**Ultraviolet** – это прототип, а далее полноценный язык со своим интерпретатором, написанный на языке программирования Rust.  

Язык имеет синтаксис, схожий с синтаксисом HTML или XML, что делает его простым для парсинга и *почти* понятным для чтения. Сам по себе язык статически строго типизируемый, что достигается строгой проверкой на типы в процессе работы фронтенда языка.

```xml
<main>
    <!-- Import external modules -->
    <import>
        <name>math</name>
    </import>
    <import>
        <name>string</name>
        <as>str</as>
    </import>

    <!-- 
        Data types
        All literals must be wrapped in their data type

        A self-closing tag (e.g. <int />) is used as a data type for typechecker.
    -->

    <!-- Strings -->
    <str>Hello world!</str>

    <!-- Integer i8, u8, ... i64, u64 -->
    <i32>255</i32>

    <!-- Floating point f32, f64 -->
    <f32>69.67</f32>

    <!-- Booleans (only 0 and 1 literals allowed) -->
    <bool>0</bool>

    <!-- Null can be used as type or as value -->
    <null />

    <!-- Union types -->
    <!-- Arguments with that type can contain multiple primitives types -->
    <union>
        <i32 />
        <null />
    </union>


    <!-- Arithmetical operations -->

    <!-- Sum can handle numerous arguments (maybe) -->
    <sum>
        <f32>10</f32>
        <f32>1.5</f32>
    </sum>

    <!-- Subtraction perform operation in passed arguments order (5 - 3) -->
    <sub>
        <i8>5</i8>
        <i8>3</i8>
    </sub>

    <!-- Mul, same as sum can handle numerous arguments -->
    <mul>
        <i8>6</i8>
        <i8>9</i8>
    </mul>

    <!-- 6 / 3 -->
    <div>
        <f32>6</f32>
        <f32>3</f32>
    </div>

    <!-- 6 % 3 -->
    <mod>
        <i8>6</i8>
        <i8>3</i8>
    </mod>

    <!-- Math operations also can be nested -->
    <!--
        In this case, the priority of mathematical operations is ignored, 
        and the calculation is performed based on the nesting of blocks ((1 + 2) * 3) 
    -->
    <mul>
        <sum>
            <i8>1</i8>
            <i8>2</i8>
        </sum>
        <i8>3</i8>
    </mul>

    <!-- Logical operations -->
    <!-- Equality -->
    <eq>
        <i8>5</i8>
        <i8>5</i8>
    </eq>

    <!-- Not Equality -->
    <neq>
        <i8>5</i8>
        <i8>8</i8>
    </neq>

    <!-- Greater then (5 > 8) Also can used as `gte` tag -->
    <gt>
        <i8>5</i8>
        <i8>8</i8>
    </gt>

    <!-- Less then (5 < 8) Also can used as `lte` tag -->
    <lt>
        <i8>5</i8>
        <i8>8</i8>
    </lt>

    <!-- Can handle numerous arguments -->
    <and>
        <bool>1</bool>
        <bool>1</bool>
    </and>

    <or>
        <bool>1</bool>
        <bool>0</bool>
    </or>

    <!-- Inverse boolean value -->
    <not>
        <bool>1</bool>
    </not>


    <!-- Var definitions -->
    <let>
        <!-- Variable name automatically casts to string -->
        <name>variable_name</name>
        <value>
            <i8>69</i8>
        </value>
    </let>

    <!-- Access variables value -->
    <variable_name />

    <!-- Variable assignment -->
    <variable_name>
        <i8>88</i8>
    </variable_name>


    <!-- Conditional operators -->
    <if>
        <test>
            <gte>
                <variable_name />
                <i8>8</i8>
            </gte>
        </test>

        <then>
            <!-- If body -->
        </then>
        <else>
            <!-- Else body -->
        </else>
    </if>


    <!-- For loops -->
    <for>
        <iter>iterator_name</iter>
        <start>
            <f32>0</f32>
        </start>
        <end>
            <f32>10</f32>
        </end>
        <!-- [OPTIONAL] Step -->
        <step>
            <f32>0.5</f32>
        </step>

        <body>
            <!-- Loop body -->
        </body>
    </for>


    <!-- While loops -->
    <let>
        <name>some_var</name>
        <value>
            <i32>0</i32>
        </value>
    </let>

    <while>
        <!-- While condition -->
        <test>
            <lt>
                <some_var />
                <i32>10<i32>
            </lt>
        </test>

        <body>
            <!-- Loop body -->
            <some_var>
                <sum>
                    <some_var />
                    <i32>1</i32>
                </sum>
            </some_var>
        </body>
    </while>


    <!-- Functions -->
    <fn>
        <!-- Function name -->
        <name>some_function</name>

        <!-- Arguments definition -->
        <arg>
            <!-- Name of argument -->
            <name>argument</name>

            <!-- Argument type -->
            <type>
                <i8 />
            </type>
        </arg>

        <!-- Second argument -->
        <arg>
            <!-- Name of argument -->
            <name>argument_2</name>

            <!-- Argument type -->
            <type>
                <i8 />
            </type>
        </arg>

        <!-- Return type -->
        <returns>
            <i8 />
        </returns>

        <!-- Function body -->

        <body>
            <!-- Return -->
            <return>
                <mul>
                    <argument />
                    <argument_2 />
                </mul>
            </return>
        </body>
    </fn>

    <!-- Function calling -->
    <!-- Function arguments are passed in the order they are defined -->
    <call some_function>
        <i8>8</i8>
        <i8>6</i8>
    </call>


    <!-- Printing to console -->
    <call println>
        <str>Hello world!</str>
    </call>

    <!-- Will print `36` -->
    <call println>
        <call some_function>
            <i8>6</i8>
            <i8>6</i8>
        </call>
    </call>
</main>


<!-- Code below — just for test -->
<ffi>
    <name>show_message_box</name>
    <dll>
        <str>user32.dll</str>
    </dll>
    <func>
        <str>MessageBoxA</str>
    </func>

    <!-- Аргументы должны соответствовать ABI из DLL -->
    <args>
        <str />
        <str />
    </args>

</ffi>

<call show_message_box>
    <str>Message Box</str>
    <str>Hello from XLL FFI!</str>
</call>
```

TODO: Arrays, try-catch expr


---
**design created by AndcoolSystems**
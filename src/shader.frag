#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec4 in_Color;

void main()
{
  o_Target = in_Color;
}
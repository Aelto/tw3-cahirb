use bitflags::bitflags;

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct ClassFlags: u32 {
    /// Class is abstract, no instance of it can be created
    const CF_Abstract                     = 1 << 0;
    /// Class is defined in C++
    const CF_Native                       = 1 << 1;
    /// Class has definition in script
    const CF_Scripted                     = 1 << 2;
    /// Class definition has been exported to C++ code
    const CF_Exported                     = 1 << 3;
    /// Class is a state class
    const CF_State                        = 1 << 4;
    /// Don't compare properties to default object on serialize
    const CF_NoDefaultObjectSerialization = 1 << 5;
    /// NEVER save or load objects of this class to ANY storage
    const CF_AlwaysTransient              = 1 << 7;
    /// Class and all derived classes should be used in editor only
    const CF_EditorOnly                   = 1 << 8;
    /// This class has one or more undefined functions
    const CF_UndefinedFunctions           = 1 << 9;
    /// Class is allowed to have states (set by scripts checked by script compiler only)
    const CF_StateMachine                 = 1 << 10;
    /// Used by the compiler to protect against multiple definitions per unit.
    const CF_ODRTag                       = 1 << 11;
  }
}

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct FunctionFlags: u32 {
    /// Function is native ( implemented in C++ )
    const FF_NativeFunction       = 1 << 0;
    /// Function is static
    const FF_StaticFunction       = 1 << 1;
    /// Function is data operator
    const FF_OperatorFunction     = 1 << 2;
    /// Function is native function that was exported to script
    const FF_ExportedFunction     = 1 << 3;
    /// Function is final and cannot be overridden in child classes
    const FF_FinalFunction        = 1 << 4;
    /// Function is special event function
    const FF_EventFunction        = 1 << 5;
    /// Function takes time to execute
    const FF_LatentFunction       = 1 << 6;
    /// Function is a state entry function
    const FF_EntryFunction        = 1 << 7;
    /// Function can be called from console
    const FF_ExecFunction         = 1 << 8;
    /// Function has no body (just a declaration)
    const FF_UndefinedBody        = 1 << 9;
    /// Function is a timer
    const FF_TimerFunction        = 1 << 10;
    /// Function can be used in Scenes
    const FF_SceneFunction        = 1 << 11;
    /// Function can be used in Quests
    const FF_QuestFunction        = 1 << 12;
    /// Function is a cleanup
    const FF_CleanupFunction      = 1 << 13;
    /// Function is private
    const FF_PrivateFunction      = 1 << 14;
    /// Function is protected
    const FF_ProtectedFunction    = 1 << 15;
    /// Function is public
    const FF_PublicFunction       = 1 << 16;
    /// Function can be attached to reward
    const FF_RewardFunction       = 1 << 17;
    /// Function is an overwrite function
    const FF_ExtReplaceFunction   = 1 << 18;
    /// Function is an add function
    const FF_ExtAddFunction       = 1 << 19;
    /// Function is a wrap function
    const FF_ExtWrapFunction      = 1 << 20;
    /// Used by the compiler to protect against multiple definitions per unit.
    const FF_ODR                  = 1 << 21;
    /// Function is scripted but stole the name of a native. Used by wrappers.
    const FF_HidingNative         = 1 << 22;
  }
}

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct PropertyFlags: u32 {
    /// Property is visible in the editor's property browser
    const PF_Editable             = 1 << 0;
    /// Property is read only
    const PF_ReadOnly             = 1 << 1;
    /// Inline property edition ( for object properties only )
    const PF_Inlined              = 1 << 2;
    /// Use this flag to grant RTTI access to the field, but prevent serializing it
    const PF_NotSerialized        = 1 << 3;
    /// Property with this flag is not cooked to final build packages
    const PF_NotCooked            = 1 << 4;
    /// Property is script property of CLASS
    const PF_Scripted             = 1 << 5;
    /// Return property of function
    const PF_FuncRetValue         = 1 << 6;
    /// Function parameter
    const PF_FuncParam            = 1 << 7;
    /// Function local variable
    const PF_FuncLocal            = 1 << 8;
    /// Function parameter that is passed by reference ( can by modified by function )
    const PF_FuncOutParam         = 1 << 9;
    /// Function parameter is optional, does not need to be specified
    const PF_FuncOptionaParam     = 1 << 10;
    /// Function parameter which evaluation can be skipped, used in native functions
    const PF_FuncSkipParam        = 1 << 11;
    /// Property is saved/loaded from config
    const PF_Config               = 1 << 12;
    /// Property was exported from C++ and can be used in script code
    const PF_Exported             = 1 << 13;
    /// Property is defined in C++
    const PF_Native               = 1 << 14;
    /// Property that will be saved to a gamesave file
    const PF_Saved                = 1 << 15;
    /// Property is private
    const PF_Private              = 1 << 16;
    /// Property is protected
    const PF_Protected            = 1 << 17;
    /// Property is public
    const PF_Public               = 1 << 18;
    /// Property is automatically bindable
    const PF_AutoBind             = 1 << 19;
    /// Failed autobind will not result in runtime script errors
    const PF_AutoBindOptional     = 1 << 20;

    const PF_AccessModifiers = Self::PF_Private.bits()
      | Self::PF_Protected.bits()
      | Self::PF_Public.bits();

  }
}
